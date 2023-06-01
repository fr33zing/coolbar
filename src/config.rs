use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use wildflower::Pattern;

use crate::{components::ConfigComponent, icons, util::UtilWidgetExt};

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Icon {
    Literal { text: String },
    Material { id: String },
}

impl ToString for Icon {
    fn to_string(&self) -> String {
        match self {
            Icon::Literal { text } => text.to_owned(),
            Icon::Material { id } => icons::material_design_icon(id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub font_family: String,
    /// Font size in px
    pub font_size_px: u16,
    pub outer_padding: String,
    pub background: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animations {
    pub enable: bool,
    pub target_fps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    pub animations: Animations,
}

const DEFAULT_MONITOR: Monitor = Monitor {
    animations: Animations {
        enable: true,
        target_fps: 60.0,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub left: Vec<String>,
    pub center: Vec<String>,
    pub right: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Per-monitor configuration indexed by the monitor's connector, e.g. "HDMI-1", "DP-1", or
    /// "eDP1" depending how your monitor is connected. Accepts wildcards.
    pub monitors: BTreeMap<String, Monitor>,

    pub theme: Theme,

    pub layout: Layout,

    pub components: BTreeMap<String, ConfigComponent>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            monitors: BTreeMap::from([("*".into(), DEFAULT_MONITOR)]),
            theme: Theme {
                font_family: "Iosevka".into(),
                font_size_px: 16,
                outer_padding: "20px".into(),
                background: "#24273A".into(), // Catppuccin Macchiato, base
            },
            layout: Layout {
                left: vec!["workspaces".into()],
                center: vec!["time".into()],
                right: vec![],
            },
            components: BTreeMap::from([
                (
                    "workspaces".into(),
                    ConfigComponent::workspaces {
                        init: crate::components::workspaces::WorkspacesInit {
                            compositor: "hyprland".into(),
                        },
                    },
                ),
                (
                    "time".into(),
                    ConfigComponent::time {
                        init: crate::components::time::TimeInit {
                            icon: Icon::Material {
                                id: "schedule".into(),
                            },
                            format: r#"%-I:%M<span alpha="50%%">:%S %p</span>"#.into(),
                        },
                    },
                ),
            ]),
        }
    }
}

impl Config {
    pub fn scss_variables(&self) -> String {
        let vars = [
            ("font_family", &self.theme.font_family),
            ("font_size", &format!("{}px", &self.theme.font_size_px)),
            ("outer_padding", &self.theme.outer_padding),
            ("background", &self.theme.background),
        ];
        vars.map(|t| format!("${}: {};", t.0, t.1)).join("\n")
    }

    pub fn monitor<T>(&self, widget: &T) -> &Monitor
    where
        T: gtk::glib::IsA<gtk::Widget>,
    {
        let connector = widget.monitor_connector();

        if let Some(monitor) = self.monitors.get(&connector) {
            monitor
        } else {
            let monitor = self
                .monitors
                .iter()
                .find(|m| Pattern::new(m.0).matches(&connector));
            if let Some(monitor) = monitor {
                monitor.1
            } else {
                &DEFAULT_MONITOR
            }
        }
    }
}

pub fn load() {
    if CONFIG.initialized() {
        panic!("config was already loaded");
    }

    let config = Config::default();
    // TODO load user config file
    CONFIG.set(config).expect("failed to store config");
}

pub fn get() -> &'static Config {
    match CONFIG.get() {
        Some(config) => &config,
        None => panic!("config was not loaded"),
    }
}
