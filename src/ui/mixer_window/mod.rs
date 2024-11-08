use gtk::prelude::{ButtonExt, GtkWindowExt, WidgetExt};
use gtk::Button;

use crate::ui::utils::get_display_type;

use super::utils::DisplayType;

mod imp;
mod wayland;
mod x11;

glib::wrapper! {
  pub struct MixerWindow(ObjectSubclass<imp::MixerWindow>)
      @extends gtk::Window, gtk::Widget,
      @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl MixerWindow {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("decorated", false)
            .property("resizable", false)
            .property("deletable", false)
            .build()
    }

    pub fn build_ui(&self) {
        let button = Button::builder()
            .label("Close")
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let window_clone = self.clone();
        button.connect_clicked(move |_| {
            println!("Close button clicked");
            window_clone.close();
        });

        self.set_child(Some(&button));
    }

    pub fn move_(&self, x: i32, y: i32) {
        match get_display_type() {
            DisplayType::Wayland => self.move_wayland(x, y),
            DisplayType::X11 => {
                if self.is_realized() {
                    self.move_x11(x, y);
                } else {
                    self.connect_realize(move |window| {
                        window.move_x11(x, y);
                    });
                }
            }
        }
    }
}
