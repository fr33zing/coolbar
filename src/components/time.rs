use std::time::Duration;

use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    gtk::{self, traits::BoxExt},
    AsyncComponentSender, Component, ComponentController, Controller,
};
use serde::{Deserialize, Serialize};
use tokio::time;
use tracing::debug;

use crate::{
    components::iconbutton::{IconButtonInit, IconButtonModel},
    config::Icon,
};

use super::iconbutton::IconButtonInput;

pub struct TimeModel {
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
    /// See [`chrono::format::strftime`] for supported escape sequences.
    pub format: String,
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

        let iconbutton = IconButtonModel::builder()
            .launch(IconButtonInit {
                class: "time".into(),
                icon: init.icon,
                text: format_time(&init.format),
                dim: false,
            })
            .detach();

        let model = TimeModel {
            format: init.format,
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
                    text: Some(format_time(&self.format)),
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

fn format_time(format: &str) -> String {
    chrono::Local::now().format(format).to_string()
}
