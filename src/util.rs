use gtk::{
    prelude::{DisplayExt, MonitorExt, SurfaceExt},
    traits::NativeExt,
};
use relm4::RelmWidgetExt;

use crate::pango_span;

pub fn dim_if(text: String, cond: bool) -> String {
    if cond {
        pango_span!(text, { alpha: "50%" })
    } else {
        text
    }
}

pub fn pad_with_dim_leading_zeros(text: String, max_length: usize) -> String {
    let zeros = "0".repeat(max_length.saturating_sub(text.len()));
    let zeros = pango_span!(zeros, { alpha: "50%" });
    format!("{zeros}{text}")
}

pub trait UtilWidgetExt {
    fn monitor_connector(&self) -> String;
}

impl<T: gtk::glib::IsA<gtk::Widget>> UtilWidgetExt for T {
    fn monitor_connector(&self) -> String {
        let surface = self
            .toplevel_window()
            .expect("widget has no toplevel window")
            .surface();
        let connector = surface
            .display()
            .monitor_at_surface(&surface)
            .expect("failed to get monitor")
            .connector()
            .expect("failed to get monitor description");

        connector.to_string()
    }
}
