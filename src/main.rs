use std::{sync::OnceLock, time::Instant};

use anyhow::{Error, Result};
use relm4::{
    component::{AsyncComponent, AsyncComponentController, AsyncController},
    gtk::{
        self,
        prelude::ApplicationExt,
        traits::{BoxExt, WidgetExt},
    },
    Component, ComponentParts, ComponentSender, RelmApp,
};
use tracing::{debug, error, info, trace, Level};
use tracing_subscriber::FmtSubscriber;

pub mod data;
pub mod dbus;

mod components;
mod config;
mod icons;
mod reducers;
mod util;

use components::{
    razer_mouse::RazerMouseModel, time::TimeModel, volume::VolumeModel, workspaces::WorkspacesModel,
};
use config::Config;

const APPLICATION_ID: &str = "none.coolbar";

static START_INSTANT: OnceLock<Instant> = OnceLock::new();

struct AppModel {
    workspaces: AsyncController<WorkspacesModel>,
    razer_mouse: AsyncController<RazerMouseModel>,
    time: AsyncController<TimeModel>,
    volume: AsyncController<VolumeModel>,
}

#[derive(Debug)]
enum AppInput {}

#[relm4::component]
impl Component for AppModel {
    type Input = AppInput;
    type Output = ();
    type CommandOutput = ();
    type Init = ();

    view! {
        gtk::Window {
            set_css_classes: &["window"],

            gtk::CenterBox {
                set_css_classes: &["bar"],

                #[wrap(Some)]
                set_start_widget = &gtk::Box {
                    set_halign: gtk::Align::Start,

                    append = model.workspaces.widget(),
                },

                #[wrap(Some)]
                set_center_widget = &gtk::Box {
                    append = model.time.widget(),
                },

                #[wrap(Some)]
                set_end_widget = &gtk::Box {
                    set_halign: gtk::Align::End,

                    append = model.razer_mouse.widget(),
                    append = model.volume.widget(),
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        debug!("initializing root component");
        initialize_window(root);

        let workspaces = WorkspacesModel::builder().launch(()).detach();
        let razer_mouse = RazerMouseModel::builder().launch(()).detach();
        let time = TimeModel::builder().launch(()).detach();
        let volume = VolumeModel::builder().launch(()).detach();

        let model = AppModel {
            workspaces,
            razer_mouse,
            time,
            volume,
        };
        let widgets = view_output!();

        let took_micros = START_INSTANT.get().unwrap().elapsed().as_micros();
        info!({ took_micros }, "finished initializing app");

        ComponentParts { model, widgets }
    }
}

fn initialize_window(window: &gtk::Window) {
    debug!("initializing window");
    gtk4_layer_shell::init_for_window(window);
    gtk4_layer_shell::set_layer(window, gtk4_layer_shell::Layer::Background);
    gtk4_layer_shell::auto_exclusive_zone_enable(window);

    gtk4_layer_shell::set_anchor(window, gtk4_layer_shell::Edge::Left, true);
    gtk4_layer_shell::set_anchor(window, gtk4_layer_shell::Edge::Right, true);
    gtk4_layer_shell::set_anchor(window, gtk4_layer_shell::Edge::Top, true);
    gtk4_layer_shell::set_anchor(window, gtk4_layer_shell::Edge::Bottom, false);
    debug!("window initialized");
}

fn load_styles(config: &Config) {
    let scss = format!(
        "{}\n{}",
        config.scss_variables(),
        include_str!("styles.scss")
    );
    let css = rsass::compile_scss(scss.as_bytes(), Default::default()).expect("valid scss");
    let css = String::from_utf8(css).unwrap();
    relm4::set_global_css(&css);
}

fn init_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .pretty()
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

fn startup(app: &gtk::Application) -> Result<()> {
    debug!("loading default config");
    let config = Config::default();

    debug!("loading default styles");
    load_styles(&config);

    debug!("passing dbus connection to reducers");
    if let Some(dbus) = app.dbus_connection() {
        dbus::DBUS_CONNECTION.set(dbus)?;
    }

    Ok(())
}

fn init() -> Result<()> {
    init_tracing();
    trace!("initialized tracing");
    info!("initializing app");

    debug!("loading config");
    config::load();

    debug!("creating app");
    let app = gtk::Application::builder()
        .application_id(APPLICATION_ID)
        .build();
    app.connect_startup(|app| {
        if let Err(err) = startup(&app) {
            handle_fatal_error(err);
        }
    });

    debug!("running app");
    RelmApp::from_app(app).run::<AppModel>(());

    Ok(())
}

fn handle_fatal_error(err: Error) {
    error!("FATAL ERROR: {err}");
}

fn main() {
    START_INSTANT
        .set(Instant::now())
        .expect("failed to set start instant");

    if let Err(err) = init() {
        handle_fatal_error(err);
    }

    debug!("exiting");
}
