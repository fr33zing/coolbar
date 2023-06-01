use relm4::gtk::traits::WidgetExt;
use relm4::Component;
use relm4::{gtk, ComponentParts, ComponentSender};

use crate::config::Icon;
use crate::util;

pub struct IconButtonModel {
    icon: String,
    text: String,
    dim: bool,
}

#[derive(Debug)]
pub struct IconButtonInput {
    pub icon: Option<Icon>,
    pub text: Option<String>,
    pub dim: Option<bool>,
}

#[derive(Debug)]
pub enum IconButtonOutput {}

pub struct IconButtonInit {
    pub icon: Icon,
    pub text: String,
    pub class: String,
    pub dim: bool,
}

#[relm4::component(pub)]
impl Component for IconButtonModel {
    type CommandOutput = ();
    type Input = IconButtonInput;
    type Output = IconButtonOutput;
    type Init = IconButtonInit;

    view! {
        #[root]
        gtk::Button {
            set_cursor_from_name: Some("pointer"),
            set_css_classes: &[&init.class],

            gtk::Box {
                gtk::Label {
                    set_css_classes: &["icon"],
                    #[watch]
                    set_markup: &util::dim_if(model.icon.to_string(), model.dim)
                },
                gtk::Label {
                    #[watch]
                    set_markup: &util::dim_if(model.text.clone(), model.dim)
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = IconButtonModel {
            icon: init.icon.to_string(),
            text: init.text,
            dim: init.dim,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        if let Some(icon) = message.icon {
            self.icon = icon.to_string();
        }
        if let Some(text) = message.text {
            self.text = text;
        }
        if let Some(dim) = message.dim {
            self.dim = dim;
        }
    }
}
