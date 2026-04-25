use gdk_x11::{
    X11Display,
    x11::xlib::{Display, Xlib},
};
use gtk::prelude::*;

pub struct X11Context {
    pub xlib: Xlib,
    pub display: *mut Display,
}

impl Default for X11Context {
    fn default() -> Self {
        let xlib = Xlib::open().unwrap();
        // Reuse GDK's existing X11 connection
        let gdk_display = gdk::Display::default().unwrap();
        let x11_display = gdk_display
            .downcast_ref::<X11Display>()
            .expect("GDK display must be X11");
        let display = unsafe { x11_display.xdisplay() };

        Self { xlib, display }
    }
}
