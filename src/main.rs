use std::{sync::OnceLock, time::Instant};

use anyhow::{Error, Result};
use clap::Parser;
use relm4::{
    gtk::{self, prelude::ApplicationExt, traits::WidgetExt},
    Component, ComponentParts, ComponentSender, RelmApp,
};
use tracing::{debug, error, info, trace, warn, Level};
use tracing_subscriber::FmtSubscriber;

pub mod data;
pub mod dbus;

pub mod args;
mod components;
mod config;
mod icons;
pub mod macros;
mod reducers;
mod util;
pub mod widgets;

use components::AppModel;

use crate::components::ConfigWidgetExt;

pub const APPLICATION_NAME: &str = "coolbar";
pub const APPLICATION_ID: &str = "none.coolbar";

#[derive(Debug)]
pub enum AppModelInput {}

#[relm4::component(pub)]
impl Component for AppModel {
    type Input = AppModelInput;
    type Output = ();
    type CommandOutput = ();
    type Init = ();

    view! {
        #[root]
        gtk::Window {
            set_css_classes: &["window"],

            gtk::CenterBox {
                set_css_classes: &["bar"],

                #[wrap(Some)]
                #[name = "left"]
                set_start_widget = &gtk::Box {
                    set_halign: gtk::Align::Start,
                },

                #[wrap(Some)]
                #[name = "center"]
                set_center_widget = &gtk::Box {
                },

                #[wrap(Some)]
                #[name = "right"]
                set_end_widget = &gtk::Box {
                    set_halign: gtk::Align::End,
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

        let mut model = AppModel::default();
        let widgets = view_output!();

        generate_components_from_config(&mut model, &widgets);

        info!("finished initializing app");

        ComponentParts { model, widgets }
    }
}

fn generate_components_from_config(app_model: &mut AppModel, widgets: &AppModelWidgets) {
    let config = config::get();
    for (area, layout, container) in [
        ("left", &config.layout.left, &widgets.left),
        ("center", &config.layout.center, &widgets.center),
        ("right", &config.layout.right, &widgets.right),
    ] {
        for name in layout {
            let Some(config) = config.components.get(name) else {
                warn!({ area, name }, "failed to find component in config");
                continue;
            };

            trace!({ area, name }, "generating component from config");
            container.generate_child_from_config(app_model, &config);
        }
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

fn load_styles() {
    let scss = format!(
        "{}\n{}",
        config::get().scss_variables(),
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
    debug!("loading default styles");
    load_styles();

    debug!("passing dbus connection to reducers");
    if let Some(dbus) = app.dbus_connection() {
        dbus::DBUS_CONNECTION.set(dbus)?;
    }

    Ok(())
}

fn init() -> Result<()> {
    debug!("parsing command line arguments");
    let args = args::Args::parse();
    if let Some(format) = args.print_default_config {
        let format = format.or(Some("yaml".into())).unwrap();
        let default_config = match format.to_lowercase().as_str() {
            "json" => serde_json::to_string_pretty(&config::Config::default())?,
            "toml" => toml::to_string(&config::Config::default())?,
            "yaml" | _ => serde_yaml::to_string(&config::Config::default())?,
        };
        println!("{default_config}");
        return Ok(());
    }

    init_tracing();
    trace!("initialized tracing");
    info!("initializing app");

    debug!("loading config");
    config::load();

    debug!("loading icon codepoints");
    icons::load_codepoints();

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
    if let Err(err) = init() {
        handle_fatal_error(err);
    }
    debug!("exiting");
}
