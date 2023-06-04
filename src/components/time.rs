use std::time::Duration;

use chrono_tz::Tz;
use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    gtk::{self, traits::BoxExt},
    AsyncComponentSender, Component, ComponentController, Controller,
};
use serde::{Deserialize, Serialize};
use tokio::time;
use tracing::{debug, warn};

use crate::{
    components::iconbutton::{IconButtonInit, IconButtonModel},
    config::Icon,
};

use super::iconbutton::IconButtonInput;

pub struct TimeModel {
    timezone: Option<Tz>,
    format: String,
    interval: time::Interval,
    iconbutton: Controller<IconButtonModel>,
}

#[derive(Debug)]
pub enum TimeInput {
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
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        debug!("initializing time component");
        let mut interval = time::interval(interval_duration(&init.format));
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

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
            .detach();

        let model = TimeModel {
            format: init.format,
            timezone,
            interval,
            iconbutton,
        };
        let widgets = view_output!();

        sender.input(TimeInput::Tick);

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>) {
        match message {
            TimeInput::Tick => {
                self.iconbutton.emit(IconButtonInput {
                    icon: None,
                    text: Some(format_time(&self.format, &self.timezone)),
                    dim: None,
                });
                self.interval.tick().await;
                sender.input(TimeInput::Tick);
            }
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
