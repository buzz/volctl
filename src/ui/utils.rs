use gdk::prelude::DisplayExtManual;

pub enum DisplayType {
    Wayland,
    X11,
}

pub fn get_display_type() -> DisplayType {
    if gdk::Display::default()
        .expect("display type could not be determined")
        .backend()
        .is_wayland()
    {
        DisplayType::Wayland
    } else {
        DisplayType::X11
    }
}
