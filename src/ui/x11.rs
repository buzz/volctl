use gdk_x11::{
    X11Display,
    x11::xlib::{Display, Xlib},
};
use gtk::prelude::*;

pub struct X11Context {
    pub display: *mut Display,
}

impl X11Context {
    /// Get the cached xlib function table (opened once, cached by the x11-dl crate).
    pub fn xlib(&self) -> Xlib {
        Xlib::open().expect("Failed to open Xlib")
    }
}

unsafe impl Send for X11Context {}
unsafe impl Sync for X11Context {}

// Raw pointer is safe to clone
impl Clone for X11Context {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for X11Context {}

impl Default for X11Context {
    fn default() -> Self {
        // Reuse GDK's existing X11 connection
        let gdk_display = gdk::Display::default().unwrap();
        let x11_display = gdk_display
            .downcast_ref::<X11Display>()
            .expect("GDK display must be X11");
        let display = unsafe { x11_display.xdisplay() };

        Self { display }
    }
}
