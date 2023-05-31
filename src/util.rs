use gtk::{
    prelude::{DisplayExt, MonitorExt, SurfaceExt},
    traits::NativeExt,
};
use relm4::RelmWidgetExt;

pub fn dim_if(s: String, cond: bool) -> String {
    if cond {
        format!("<span alpha=\"50%\">{s}</span>")
    } else {
        s
    }
}

pub fn dim_leading_zeros(s: String) -> String {
    if !s.starts_with("0") {
        return s;
    }

    let mut zeros: usize = 0;
    for c in s.chars() {
        if c == '0' {
            zeros += 1;
        } else {
            break;
        }
    }

    let (zeros, rest) = s.split_at(zeros);
    format!("<span alpha=\"50%\">{zeros}</span>{rest}")
}

pub fn pad_with_dim_leading_zeros(s: String, max_length: usize) -> String {
    let zeros = "0".repeat(max_length.saturating_sub(s.len()));
    format!("<span alpha=\"50%\">{zeros}</span>{s}")
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
