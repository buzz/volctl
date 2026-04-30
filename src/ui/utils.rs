use gdk::prelude::DisplayExtManual;

use crate::errors::X11Error;

#[derive(Debug, PartialEq)]
pub enum DisplayType {
    Wayland,
    X11,
}

pub fn get_display_type() -> Result<DisplayType, X11Error> {
    let display = gdk::Display::default().ok_or(X11Error::NoDisplay)?;
    Ok(if display.backend().is_wayland() {
        DisplayType::Wayland
    } else {
        DisplayType::X11
    })
}
