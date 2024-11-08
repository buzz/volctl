use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use glib::subclass::object::{ObjectImpl, ObjectImplExt};
use glib::subclass::types::{ObjectSubclass, ObjectSubclassExt};
use gtk::prelude::{GtkWindowExt, WidgetExt};
use gtk::subclass::widget::WidgetImplExt;
use gtk::subclass::{widget::WidgetImpl, window::WindowImpl};
use gtk::{Box, Orientation};

use crate::ui::utils::{get_display_type, DisplayType};

use super::constants::COL_SPACING;
use super::scale::VolumeScale;

pub struct MixerWindow {
    pub(super) box_: Rc<RefCell<Box>>,
    // Stores scale widgets by stream index
    pub(super) sinks: Rc<RefCell<HashMap<u32, VolumeScale>>>,
    pub(super) sink_inputs: Rc<RefCell<HashMap<u32, VolumeScale>>>,
}

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

        obj.set_child(Some(&*self.box_.borrow()));
        obj.set_visible(false);
    }
}

impl WindowImpl for MixerWindow {}

impl WidgetImpl for MixerWindow {
    fn realize(&self) {
        self.parent_realize();

        if get_display_type() == DisplayType::X11 {
            self.obj().realize_x11();
        }
    }
}

impl Default for MixerWindow {
    fn default() -> Self {
        Self {
            box_: Rc::from(RefCell::from(
                Box::builder()
                    .orientation(Orientation::Horizontal)
                    .homogeneous(true)
                    .spacing(COL_SPACING)
                    .build(),
            )),
            sinks: Rc::from(RefCell::from(HashMap::new())),
            sink_inputs: Rc::from(RefCell::from(HashMap::new())),
        }
    }
}
