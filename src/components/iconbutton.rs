use relm4::gtk::traits::WidgetExt;
use relm4::Component;
use relm4::{gtk, ComponentParts, ComponentSender};

use crate::icons::material_design_icon;
use crate::util;

pub struct IconButtonModel {
    icon: String,
    text: String,
    dim: bool,
}

#[derive(Debug)]
pub enum IconButtonInput {
    UpdateText(String),
    UpdateIcon(String),
    Update(String, String, bool),
    Dim(bool),
}

#[derive(Debug)]
pub enum IconButtonOutput {}

pub struct IconButtonInit {
    pub icon: String,
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
                    set_markup: &util::dim_if(material_design_icon(&model.icon), model.dim)
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
            icon: init.icon,
            text: init.text,
            dim: init.dim,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            IconButtonInput::UpdateIcon(icon) => self.icon = icon,
            IconButtonInput::UpdateText(text) => self.text = text,
            IconButtonInput::Update(icon, text, dim) => {
                self.icon = icon;
                self.text = text;
                self.dim = dim;
            }
            IconButtonInput::Dim(dim) => self.dim = dim,
        };
    }
}
