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
    iconbutton: Controller<IconButtonModel>,
}

#[derive(Debug)]
pub enum RazerMouseInput {
    Update(bool, f64),
}

#[derive(Debug)]
pub enum RazerMouseOutput {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RazerMouseInit {
    pub icon: Icon,
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
        OPENRAZER.subscribe(&tx, |msg| {
            RazerMouseInput::Update(msg.mouse_detected, msg.mouse_battery)
        });
        relm4::spawn(async move {
            while let Some(msg) = rx.recv().await {
                sender.input(msg);
            }
        });

        let iconbutton = IconButtonModel::builder()
            .launch(IconButtonInit {
                class: "mouse".into(),
                icon: init.icon,
                text: "???%".into(),
                dim: true,
            })
            .detach();

        let model = RazerMouseModel { iconbutton };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            RazerMouseInput::Update(detected, battery) => {
                let text = if detected {
                    let battery = f64::round(battery);
                    util::pad_with_dim_leading_zeros(format!("{battery}%"), 4)
                } else {
                    "Not detected".into()
                };

                self.iconbutton.emit(IconButtonInput {
                    icon: None,
                    text: Some(text),
                    dim: Some(!detected),
                });
            }
        };
    }
}
