use gtk::traits::{ButtonExt, PopoverExt, WidgetExt};
use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    gtk, AsyncComponentSender,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{config::Icon, widgets::panel::Panel};

pub struct PowerModel {
    icon: Icon,
    panel_open: bool,
}

#[derive(Debug)]
pub enum PowerInput {
    PanelOpen(bool),
}

#[derive(Debug)]
pub enum PowerOutput {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerInit {
    pub icon: Icon,
}

impl Default for PowerInit {
    fn default() -> Self {
        Self {
            icon: Icon::Material {
                id: "power_settings_new".into(),
            },
        }
    }
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for PowerModel {
    type Input = PowerInput;
    type Output = PowerOutput;
    type Init = PowerInit;

    view! {
        #[root]
        gtk::Box {
            gtk::Button {
                set_cursor_from_name: Some("pointer"),
                set_css_classes: &["power"],

                gtk::Box {
                    gtk::Label {
                        set_css_classes: &["icon"],
                        set_markup: &model.icon.to_string(),
                    },
                }
            },

            Panel {
                #[watch]
                set_open: model.panel_open,
                connect_closed[sender] => move |_| {
                  sender.input(PowerInput::PanelOpen(false));
                },

                gtk::Label {
                    set_text: "Hello world!"
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        debug!("initializing power component");

        let model = PowerModel {
            icon: init.icon,
            panel_open: false,
        };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            PowerInput::PanelOpen(open) => self.panel_open = open,
        }
    }
}
