mod imp;

use glib::Object;
use gtk::{
    glib,
    traits::{PopoverExt, WidgetExt},
};
use relm4::{ContainerChild, RelmSetChildExt};

glib::wrapper! {
    pub struct Panel(ObjectSubclass<imp::Panel>)
        @extends gtk::Popover, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Panel {
    pub fn new() -> Self {
        Object::builder().build()
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainerChild for Panel {
    type Child = gtk::Widget;
}

impl RelmSetChildExt for Panel {
    fn container_set_child(&self, widget: Option<&impl AsRef<gtk::Widget>>) {
        self.set_child(widget.map(|w| w.as_ref()));
    }

    fn container_get_child(&self) -> Option<gtk::Widget> {
        self.child()
    }
}

impl Panel {
    pub fn set_open(&self, open: bool) {
        if open {
            self.popup();
            self.set_css_classes(&["visible"]);
        } else {
            self.popdown();
            self.set_css_classes(&[]);
        }
    }
}
