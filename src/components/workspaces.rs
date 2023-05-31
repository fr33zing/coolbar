use std::{
    f64::consts::PI,
    time::{Duration, Instant},
};

use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    drawing::DrawHandler,
    gtk::{
        self,
        cairo::{self, LineCap, Operator},
        traits::{DrawingAreaExt, WidgetExt},
    },
    AsyncComponentSender,
};
use tokio::{
    task,
    time::{self, MissedTickBehavior},
};
use tracing::debug;

use crate::{
    config,
    data::wayland_compositor::WaylandCompositor,
    reducers::hyprland::{HyprlandReducer, REDUCER as HYPRLAND},
    util::UtilWidgetExt,
};

const FAST_INTERPOLATION: Duration = Duration::from_millis(150);
const SLOW_INTERPOLATION: Duration = Duration::from_millis(400);

pub struct WorkspacesModel {
    /// Connector of the monitor on which this component exists.
    monitor_connector: String,

    /// Indicates whether or not the next frame should be drawn.
    drawing: bool,

    last_update: Instant,
    hyprland: Option<HyprlandReducer>,
    handler: DrawHandler,
    width: f64,
    height: f64,
    fast_interpolation: Duration,
    slow_interpolation: Duration,
    dot_fast_x: f64,
    dot_fast_x_start: f64,
    dot_slow_x: f64,
    dot_slow_x_start: f64,
}

#[derive(Debug)]
pub enum WorkspacesInput {
    Update(HyprlandReducer),
    Resize((i32, i32)),
    Draw,
}

#[derive(Debug)]
pub enum Output {}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for WorkspacesModel {
    type Input = WorkspacesInput;
    type Output = Output;
    type Init = ();

    view! {
        #[root]
        gtk::Button {
            set_cursor_from_name: Some("pointer"),
            set_css_classes: &["workspaces"],

            #[local_ref]
            area -> gtk::DrawingArea {
                set_width_request: (config::get().theme.font_size * 14).into(),

                connect_resize[sender] => move |_, x, y| {
                    sender.input(WorkspacesInput::Resize((x, y)));
                }
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        debug!("initializing workspaces component");

        // Connect to Hyprland
        let (tx, rx) = relm4::channel::<WorkspacesInput>();
        HYPRLAND.subscribe(&tx, |data| WorkspacesInput::Update(data.clone()));
        let sender_clone = sender.clone();
        task::spawn(async move {
            while let Some(data) = rx.recv().await {
                sender_clone.input(data);
            }
        });

        // Begin drawing
        let monitor_config = config::get().monitor(&root);
        let sender_clone = sender.clone();
        task::spawn(async move {
            let target_fps = monitor_config.animations.target_fps;
            let target_frame_time = Duration::from_micros((1.0 / target_fps * 1_000_000.0) as u64);
            let mut interval = time::interval(target_frame_time);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                sender_clone.input(WorkspacesInput::Draw);
                interval.tick().await;
            }
        });

        let model = WorkspacesModel {
            monitor_connector: root.monitor_connector(),
            drawing: true,
            last_update: Instant::now(),
            hyprland: None,
            handler: DrawHandler::new(),
            width: 0.0,
            height: 0.0,
            fast_interpolation: if monitor_config.animations.enable {
                FAST_INTERPOLATION
            } else {
                // HACK this is a bad way to disable animations, if they were disabled properly the
                // draw loop thread could be avoided when animations are disabled.
                Duration::from_secs(0)
            },
            slow_interpolation: if monitor_config.animations.enable {
                SLOW_INTERPOLATION
            } else {
                Duration::from_secs(0)
            },
            dot_fast_x: 0.0,
            dot_fast_x_start: 0.0,
            dot_slow_x: 0.0,
            dot_slow_x_start: 0.0,
        };
        let area = model.handler.drawing_area();
        let widgets = view_output!();

        config::get().monitor(&root);
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>) {
        match message {
            WorkspacesInput::Update(data) => {
                self.last_update = Instant::now();
                self.drawing = true;
                self.hyprland = Some(data);
                self.dot_fast_x_start = self.dot_fast_x;
                self.dot_slow_x_start = self.dot_slow_x;

                sender.input(WorkspacesInput::Draw);
            }
            WorkspacesInput::Resize((x, y)) => {
                self.width = x as f64;
                self.height = y as f64;
            }
            WorkspacesInput::Draw => {
                let ctx = self.handler.get_context();
                self.draw(&ctx);
            }
        };
    }
}

impl WorkspacesModel {
    fn clear(&self, ctx: &cairo::Context) {
        ctx.set_operator(Operator::Clear);
        ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        ctx.paint().expect("Couldn't clear context");
    }

    fn draw(&mut self, ctx: &cairo::Context) {
        if !self.drawing {
            return;
        };
        let Some(hyprland) = &self.hyprland else {
            return
        };

        // Calculate placement of circles
        let thickness = 2.0;
        let spacing = 6.0;
        let horizontal_margin = spacing * 2.0;
        let n_circles = 10.0;
        let n_spaces = n_circles - 1.0;
        let diameter = (self.width
            - (spacing * n_spaces)
            - (thickness * n_circles)
            - (horizontal_margin * 2.0))
            / n_circles;
        let radius = diameter / 2.0;
        let offset_per_workspace = spacing + thickness + diameter;
        let initial_offset = radius + (thickness / 2.0) + horizontal_margin;
        let y = self.height / 2.0;
        let x = |i: usize| initial_offset + (offset_per_workspace * (i as f64));

        // Stop drawing if interpolation is finished
        let time = self.last_update.elapsed();
        self.drawing = time < self.slow_interpolation;

        // Get color from stylesheet
        let color = self.handler.drawing_area().color();
        let (red, green, blue, alpha) = (
            color.red() as f64,
            color.green() as f64,
            color.blue() as f64,
            color.alpha() as f64,
        );

        // Begin drawing
        self.clear(&ctx);
        ctx.set_operator(Operator::Source);

        // Workspaces
        {
            ctx.set_line_width(thickness);

            for i in 0..(n_circles as usize) {
                let empty = if let Some(ws) = hyprland.workspaces().get(&i) {
                    hyprland.workspace_is_empty(ws)
                } else {
                    true
                };
                let alpha = alpha * if empty { 0.5 } else { 1.0 };

                ctx.set_source_rgba(red, green, blue, alpha);
                ctx.arc(x(i), y, radius, 0.0, std::f64::consts::PI * 2.0);
                ctx.stroke().expect("couldn't stroke arc");
            }
        }

        // Active workspace
        {
            let monitor = hyprland
                .monitors()
                .values()
                .find(|m| m.connector == self.monitor_connector);

            let Some(monitor) = monitor else {
                return;
            };
            let Some(active_workspace) = hyprland.active_workspace(monitor) else {
                return;
            };

            let x_dest = x(active_workspace.id);
            let radius = radius - thickness * 1.5;

            if self.dot_slow_x < x(0) {
                // Set initial positions to prevent dot coming in from the left side when the
                // animation first begins.
                self.dot_fast_x = x_dest;
                self.dot_fast_x_start = x_dest;
                self.dot_slow_x = x_dest;
                self.dot_slow_x_start = x_dest;
            } else {
                self.dot_fast_x = if time < self.fast_interpolation {
                    ease_out_sine(
                        time,
                        self.dot_fast_x_start,
                        x_dest - self.dot_fast_x_start,
                        self.fast_interpolation,
                    )
                } else {
                    x_dest
                };
                self.dot_slow_x = if time < self.slow_interpolation {
                    ease_out_sine(
                        time,
                        self.dot_slow_x_start,
                        x_dest - self.dot_slow_x_start,
                        self.slow_interpolation,
                    )
                } else {
                    x_dest
                };
            }

            ctx.set_source_rgba(red, green, blue, alpha);
            ctx.set_line_width(radius * 2.0);
            ctx.set_line_cap(LineCap::Round);
            ctx.move_to(self.dot_fast_x, y);
            ctx.line_to(self.dot_slow_x, y);
            ctx.stroke().expect("couldn't stroke arc");
        }
    }
}

// into
fn ease_out_sine(elapsed: Duration, start: f64, change: f64, duration: Duration) -> f64 {
    // TODO replace with Duration::div_duration_f64 once div_duration is stabilized (#63139)
    change * (elapsed.as_secs_f64() / duration.as_secs_f64() * (PI / 2.0)).sin() + start
}
