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
    icon: Icon,
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
        _init: Self::Init,
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
        let iconbutton = IconButtonModel::builder()
            .launch(IconButtonInit {
                class: "volume".into(),
                icon: Icon::Material {
                    id: "volume_off".into(),
                },
                text: "???%".into(),
                dim: true,
            })
            .detach();

        let model = VolumeModel { iconbutton };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            VolumeInput::Update(volume, muted) => {
                let icon = if muted { "volume_off" } else { "volume_up" };
                let text = util::pad_with_dim_leading_zeros(format!("{volume}%"), 4);

                self.iconbutton.emit(IconButtonInput {
                    icon: Some(Icon::Material { id: icon.into() }),
                    text: Some(text),
                    dim: Some(muted),
                });
            }
        };
    }
}
