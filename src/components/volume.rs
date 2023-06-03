use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    gtk::{self, traits::BoxExt},
    AsyncComponentSender, Component, ComponentController, Controller,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    components::iconbutton::{IconButtonInit, IconButtonInput, IconButtonModel},
    config::Icon,
    reducers::pulseaudio::REDUCER as PULSEAUDIO,
    util,
};

pub struct VolumeModel {
    icon: Icon,
    icon_muted: Icon,
    iconbutton: Controller<IconButtonModel>,
}

#[derive(Debug)]
pub enum VolumeInput {
    Update(f32, bool),
}

#[derive(Debug)]
pub enum VolumeOutput {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInit {
    pub icon: Icon,
    pub icon_muted: Icon,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for VolumeModel {
    type Input = VolumeInput;
    type Output = VolumeOutput;
    type Init = VolumeInit;

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
        let (tx, rx) = relm4::channel();
        PULSEAUDIO.subscribe(&tx, |msg| VolumeInput::Update(msg.volume, msg.muted));
        relm4::spawn(async move {
            while let Some(msg) = rx.recv().await {
                sender.input(msg);
            }
        });

        debug!("initializing volume component");
        let VolumeInit { icon, icon_muted } = init;
        let iconbutton = IconButtonModel::builder()
            .launch(IconButtonInit {
                class: "volume".into(),
                icon: icon.clone(),
                text: "???%".into(),
                dim: true,
            })
            .detach();

        let model = VolumeModel {
            icon,
            icon_muted,
            iconbutton,
        };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            VolumeInput::Update(volume, muted) => {
                let icon = if muted {
                    self.icon_muted.clone()
                } else {
                    self.icon.clone()
                };
                let text = util::pad_with_dim_leading_zeros(format!("{volume}%"), 4);

                self.iconbutton.emit(IconButtonInput {
                    icon: Some(icon),
                    text: Some(text),
                    dim: Some(muted),
                });
            }
        };
    }
}
