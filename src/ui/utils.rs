use gdk::prelude::DisplayExtManual;

pub fn is_wayland() -> bool {
    gdk::Display::default()
        .map(|display| display.backend().is_wayland())
        .unwrap_or(false)
}
