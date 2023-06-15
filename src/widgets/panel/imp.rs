use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::traits::PopoverExt;

#[derive(Default)]
pub struct Panel;

#[glib::object_subclass]
impl ObjectSubclass for Panel {
    const NAME: &'static str = "Panel";
    type Type = super::Panel;
    type ParentType = gtk::Popover;
}

impl ObjectImpl for Panel {
    fn constructed(&self) {
        self.parent_constructed();
        self.obj().set_autohide(true);
        self.obj().set_has_arrow(false);
    }
}

impl WidgetImpl for Panel {}

impl PopoverImpl for Panel {
    fn activate_default(&self) {
        self.parent_activate_default()
    }

    fn closed(&self) {
        self.parent_closed()
    }
}
