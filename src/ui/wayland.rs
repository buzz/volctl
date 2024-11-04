use gtk::prelude::DisplayExtManual;

pub fn is_wayland_display() -> bool {
    gdk::Display::default()
        .map(|display| display.backend().is_wayland())
        .unwrap_or(false)
}
