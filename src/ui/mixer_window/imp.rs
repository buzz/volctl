use glib::subclass::object::{ObjectImpl, ObjectImplExt};
use glib::subclass::types::{ObjectSubclass, ObjectSubclassExt};
use gtk::prelude::WidgetExt;
use gtk::subclass::widget::WidgetImplExt;
use gtk::subclass::{widget::WidgetImpl, window::WindowImpl};

use crate::ui::utils::{get_display_type, DisplayType};

#[derive(Debug, Default)]
pub struct MixerWindow {}

#[glib::object_subclass]
impl ObjectSubclass for MixerWindow {
    const NAME: &'static str = "VolctlMixerWindow";
    type Type = super::MixerWindow;
    type ParentType = gtk::Window;
}

impl ObjectImpl for MixerWindow {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        obj.set_visible(false);
    }
}

impl WindowImpl for MixerWindow {}

impl WidgetImpl for MixerWindow {
    fn realize(&self) {
        self.parent_realize();

        match get_display_type() {
            DisplayType::X11 => {
                self.obj().realize_x11();
            }
            _ => {}
        }
    }
}
