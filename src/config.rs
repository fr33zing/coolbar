use std::{collections::BTreeMap, time::Duration};

use rand::{rngs::SmallRng, Rng};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use wildflower::Pattern;

use crate::{
    components::{
        razer_mouse::RazerMouseInit, time::TimeInit, volume::VolumeInit,
        workspaces::WorkspacesInit, ConfigComponent,
    },
    icons,
    util::UtilWidgetExt,
};

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Icon {
    Literal { text: String },
    Multiple { icons: Vec<Icon> },
    Material { id: String },
}

impl ToString for Icon {
    fn to_string(&self) -> String {
        match self {
            Icon::Literal { text } => text.to_owned(),
            Icon::Multiple { icons } => icons
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(""),
            Icon::Material { id } => icons::material_design_icon(id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PollingRate {
    Constant {
        #[serde(with = "humantime_serde")]
        interval: Duration,
    },
    VariedByRatio {
        #[serde(with = "humantime_serde")]
        interval: Duration,
        variance: f64,
    },
    VariedByDuration {
        #[serde(with = "humantime_serde")]
        interval: Duration,
        #[serde(with = "humantime_serde")]
        variance: Duration,
    },
}

impl PollingRate {
    pub fn to_duration(&self, rng: &mut SmallRng) -> Duration {
        const MIN_INTERVAL: Duration = Duration::from_micros(1);
        let mut random = || (rng.gen::<f64>() - 0.5) * 2.0; // -1 to 1

        let result = match self {
            PollingRate::Constant { interval } => *interval,
            PollingRate::VariedByRatio { interval, variance } => {
                let variance = variance.clamp(0.0, 1.0);
                interval.mul_f64(1.0 + variance * random())
            }
            PollingRate::VariedByDuration { interval, variance } => {
                let offset = variance.mul_f64(random());
                interval.saturating_add(offset)
            }
        };

        Duration::max(result, MIN_INTERVAL)
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
pub struct Providers {
    pub wayland: Wayland,
    pub openrazer: OpenRazer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wayland {
    // TODO add support for another compositor, this currently does nothing
    pub compositor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRazer {
    pub polling_rate: PollingRate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Per-monitor configuration indexed by the monitor's connector, e.g. "HDMI-1", "DP-1", or
    /// "eDP1" depending how your monitor is connected. Accepts wildcards.
    pub monitors: BTreeMap<String, Monitor>,

    pub theme: Theme,

    pub layout: Layout,

    pub providers: Providers,

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
                right: vec!["razer_mouse".into(), "volume".into()],
            },
            providers: Providers {
                wayland: Wayland {
                    compositor: "hyprland".into(),
                },
                openrazer: OpenRazer {
                    polling_rate: PollingRate::VariedByRatio {
                        interval: Duration::from_secs(2),
                        variance: 0.25,
                    },
                },
            },
            components: BTreeMap::from([
                (
                    "workspaces".into(),
                    ConfigComponent::workspaces {
                        init: WorkspacesInit {},
                    },
                ),
                (
                    "time".into(),
                    ConfigComponent::time {
                        init: TimeInit {
                            icon: Icon::Material {
                                id: "schedule".into(),
                            },
                            timezone: None,
                            format: r#"%-I:%M<span alpha="50%%">:%S %p</span>"#.into(),
                        },
                    },
                ),
                (
                    "razer_mouse".into(),
                    ConfigComponent::razer_mouse {
                        init: RazerMouseInit {
                            icon: Icon::Material { id: "mouse".into() },
                            icon_charging: Icon::Multiple {
                                icons: vec![
                                    Icon::Material { id: "mouse".into() },
                                    Icon::Material { id: "bolt".into() },
                                ],
                            },
                        },
                    },
                ),
                (
                    "volume".into(),
                    ConfigComponent::volume {
                        init: VolumeInit {
                            icon: Icon::Material {
                                id: "volume_up".into(),
                            },
                            icon_muted: Icon::Material {
                                id: "volume_off".into(),
                            },
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
