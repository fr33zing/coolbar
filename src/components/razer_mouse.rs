use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    gtk::{self, traits::BoxExt},
    AsyncComponentSender, Component, ComponentController, Controller,
};
use tracing::debug;

use super::iconbutton::{IconButtonInit, IconButtonInput};
use crate::reducers::openrazer::REDUCER as OPENRAZER;
use crate::{components::iconbutton::IconButtonModel, util};

pub struct RazerMouseModel {
    iconbutton: Controller<IconButtonModel>,
}

#[derive(Debug)]
pub enum RazerMouseInput {
    Update(bool, f64),
}

#[derive(Debug)]
pub enum RazerMouseOutput {}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for RazerMouseModel {
    type Input = RazerMouseInput;
    type Output = RazerMouseOutput;
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
        let (tx, rx) = relm4::channel();
        OPENRAZER.subscribe(&tx, |msg| {
            RazerMouseInput::Update(msg.mouse_detected, msg.mouse_battery)
        });
        relm4::spawn(async move {
            while let Some(msg) = rx.recv().await {
                sender.input(msg);
            }
        });

        debug!("initializing razer mouse component");
        let iconbutton = IconButtonModel::builder()
            .launch(IconButtonInit {
                class: "mouse".into(),
                icon: "mouse".into(),
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
                self.iconbutton
                    .emit(IconButtonInput::Update("mouse".into(), text, !detected));
            }
        };
    }
}
