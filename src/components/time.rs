use std::time::Duration;

use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    gtk::{self, traits::BoxExt},
    AsyncComponentSender, Component, ComponentController, Controller,
};
use tokio::time;
use tracing::debug;

use crate::components::iconbutton::IconButtonModel;

use super::iconbutton::{IconButtonInit, IconButtonInput};

pub struct TimeModel {
    interval: time::Interval,
    iconbutton: Controller<IconButtonModel>,
}

#[derive(Debug)]
pub enum TimeInput {
    Tick,
}

#[derive(Debug)]
pub enum Output {}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for TimeModel {
    type Input = TimeInput;
    type Output = Output;
    type Init = ();

    view! {
        #[root]
        gtk::Box {
            append: model.iconbutton.widget(),
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        debug!("initializing time component");
        let mut interval = time::interval(Duration::from_secs(1));
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        let iconbutton = IconButtonModel::builder()
            .launch(IconButtonInit {
                class: "time".into(),
                icon: "schedule".into(),
                text: format_time(),
                dim: false,
            })
            .detach();

        let model = TimeModel {
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
                self.iconbutton
                    .emit(IconButtonInput::UpdateText(format_time()));
                self.interval.tick().await;
                sender.input(TimeInput::Tick);
            }
        }
    }
}

fn format_time() -> String {
    chrono::Local::now()
        .format(r#"%-I:%M<span alpha="50%%">:%S %p</span>"#)
        .to_string()
}
