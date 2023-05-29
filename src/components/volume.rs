use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    gtk::{self, traits::BoxExt},
    AsyncComponentSender, Component, ComponentController, Controller,
};

use super::iconbutton::{IconButtonInit, IconButtonInput};
use crate::components::iconbutton::IconButtonModel;
use crate::reducers::pulseaudio::REDUCER as PULSEAUDIO;
use crate::util;

pub struct VolumeModel {
    iconbutton: Controller<IconButtonModel>,
}

#[derive(Debug)]
pub enum VolumeInput {
    Update(f32, bool),
}

#[derive(Debug)]
pub enum VolumeOutput {}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for VolumeModel {
    type Input = VolumeInput;
    type Output = VolumeOutput;
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
        PULSEAUDIO.subscribe(&tx, |msg| VolumeInput::Update(msg.volume, msg.muted));
        relm4::spawn(async move {
            while let Some(msg) = rx.recv().await {
                sender.input(msg);
            }
        });

        tracing::debug!("initializing volume component");
        let iconbutton = IconButtonModel::builder()
            .launch(IconButtonInit {
                class: "volume".into(),
                icon: "volume_off".into(),
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

                self.iconbutton
                    .emit(IconButtonInput::Update(icon.into(), text, muted));
            }
        };
    }
}
