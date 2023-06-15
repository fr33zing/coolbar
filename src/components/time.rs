use std::time::Duration;

use chrono_tz::Tz;
use gtk::traits::PopoverExt;
use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    gtk::{self, traits::BoxExt},
    AsyncComponentSender, Component, ComponentController, Controller,
};
use serde::{Deserialize, Serialize};
use tokio::{task, time};
use tracing::{debug, warn};

use crate::{
    components::iconbutton::{IconButtonInit, IconButtonModel, IconButtonOutput},
    config::Icon,
    widgets::panel::Panel,
};

use super::iconbutton::IconButtonInput;

pub struct TimeModel {
    panel_open: bool,
    timezone: Option<Tz>,
    format: String,
    iconbutton: Controller<IconButtonModel>,
}

#[derive(Debug)]
pub enum TimeInput {
    PanelOpen(bool),
    Tick,
}

#[derive(Debug)]
pub enum Output {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeInit {
    pub icon: Icon,
    /// See https://docs.rs/chrono-tz/latest/chrono_tz/#modules for timezones.
    pub timezone: Option<String>,
    /// See [`chrono::format::strftime`] for supported escape sequences.
    pub format: String,
}

impl Default for TimeInit {
    fn default() -> Self {
        Self {
            icon: Icon::Material {
                id: "schedule".into(),
            },
            timezone: None,
            format: r#"%-I:%M<span alpha="50%%">:%S %p</span>"#.into(),
        }
    }
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for TimeModel {
    type Input = TimeInput;
    type Output = Output;
    type Init = TimeInit;

    view! {
        #[root]
        gtk::Box {
            append: model.iconbutton.widget(),

            Panel {
                #[watch]
                set_open: model.panel_open,
                connect_closed[sender] => move |_| {
                  sender.input(TimeInput::PanelOpen(false));
                },

                gtk::Label {
                    set_text: "Hello world!"
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        debug!("initializing time component");

        {
            let sender = sender.clone();
            let interval_duration = interval_duration(&init.format);
            task::spawn(async move {
                let mut interval = time::interval(interval_duration);
                interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
                loop {
                    interval.tick().await;
                    sender.input(TimeInput::Tick);
                }
            });
        }

        let timezone: Option<Tz> = if let Some(timezone) = init.timezone {
            match timezone.parse() {
                Ok(timezone) => Some(timezone),
                Err(err) => {
                    warn!({ timezone, err }, "failed to parse timezone");
                    None
                }
            }
        } else {
            None
        };

        let iconbutton = IconButtonModel::builder()
            .launch(IconButtonInit {
                class: "time".into(),
                icon: init.icon,
                text: format_time(&init.format, &timezone),
                dim: false,
            })
            .forward(sender.input_sender(), |o| match o {
                IconButtonOutput::Clicked => TimeInput::PanelOpen(true),
            });

        let model = TimeModel {
            panel_open: false,
            format: init.format,
            timezone,
            iconbutton,
        };
        let widgets = view_output!();

        sender.input(TimeInput::Tick);

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            TimeInput::PanelOpen(open) => self.panel_open = open,
            TimeInput::Tick => self.iconbutton.emit(IconButtonInput {
                icon: None,
                text: Some(format_time(&self.format, &self.timezone)),
                dim: None,
            }),
        }
    }
}

fn interval_duration(format: &str) -> Duration {
    let seconds_escapes = ["%S", "%-S", "%_S", "%0S"];
    if seconds_escapes.iter().any(|s| format.contains(s)) {
        Duration::from_secs(1)
    } else {
        Duration::from_secs(60)
    }
}

fn format_time(format: &str, timezone: &Option<Tz>) -> String {
    match timezone {
        Some(timezone) => chrono::Utc::now()
            .with_timezone(timezone)
            .format(format)
            .to_string(),
        None => chrono::Local::now().format(format).to_string(),
    }
}
