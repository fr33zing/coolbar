use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    gtk::{self, traits::BoxExt},
    AsyncComponentSender, Component, ComponentController, Controller,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    components::iconbutton::{IconButtonInit, IconButtonModel},
    config::Icon,
    reducers::openrazer::REDUCER as OPENRAZER,
    util,
};

use super::iconbutton::IconButtonInput;

pub struct RazerMouseModel {
    icon: Icon,
    icon_charging: Icon,
    iconbutton: Controller<IconButtonModel>,
}

#[derive(Debug)]
pub enum RazerMouseInput {
    Update {
        detected: bool,
        charging: bool,
        battery_level: f64,
    },
}

#[derive(Debug)]
pub enum RazerMouseOutput {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RazerMouseInit {
    pub icon: Icon,
    pub icon_charging: Icon,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for RazerMouseModel {
    type Input = RazerMouseInput;
    type Output = RazerMouseOutput;
    type Init = RazerMouseInit;

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
        debug!("initializing razer mouse component");

        let (tx, rx) = relm4::channel();
        OPENRAZER.subscribe(&tx, |msg| RazerMouseInput::Update {
            detected: msg.mouse_detected,
            charging: msg.mouse_charging,
            battery_level: msg.mouse_battery_level,
        });
        relm4::spawn(async move {
            while let Some(msg) = rx.recv().await {
                sender.input(msg);
            }
        });

        let iconbutton = IconButtonModel::builder()
            .launch(IconButtonInit {
                class: "mouse".into(),
                icon: init.icon.clone(),
                text: "???%".into(),
                dim: true,
            })
            .detach();

        let model = RazerMouseModel {
            icon: init.icon,
            icon_charging: init.icon_charging,
            iconbutton,
        };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            RazerMouseInput::Update {
                detected,
                charging,
                battery_level,
            } => {
                let icon = if charging {
                    &self.icon_charging
                } else {
                    &self.icon
                };

                let text = if detected {
                    let battery = f64::round(battery_level);
                    util::pad_with_dim_leading_zeros(format!("{battery}%"), 4)
                } else {
                    "Not detected".into()
                };

                self.iconbutton.emit(IconButtonInput {
                    icon: Some(icon.clone()),
                    text: Some(text),
                    dim: Some(!detected),
                });
            }
        };
    }
}
