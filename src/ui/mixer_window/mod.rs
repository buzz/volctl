use std::{cell::RefCell, collections::HashMap, rc::Rc};

use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::{BoxExt, WidgetExt};

use super::utils::{get_display_type, DisplayType};
use crate::pulse::StreamData;
use scale::VolumeScale;

mod constants;
mod imp;
mod scale;
mod wayland;
mod x11;

glib::wrapper! {
  pub struct MixerWindow(ObjectSubclass<imp::MixerWindow>)
      @extends gtk::Window, gtk::Widget,
      @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl MixerWindow {
    pub fn new() -> Self {
        let window: MixerWindow = glib::Object::builder()
            .property("decorated", false)
            .property("resizable", false)
            .property("deletable", false)
            .build();

        window
    }

    pub fn update_sinks(&self, sink_streams: &HashMap<u32, StreamData>) {
        self.update_volume_scales(sink_streams, self.imp().sinks.clone());
    }

    pub fn update_sink_inputs(&self, sink_input_streams: &HashMap<u32, StreamData>) {
        self.update_volume_scales(sink_input_streams, self.imp().sink_inputs.clone());
    }

    fn update_volume_scales(
        &self,
        streams: &HashMap<u32, StreamData>,
        scales: Rc<RefCell<HashMap<u32, VolumeScale>>>,
    ) {
        let imp = self.imp();
        let box_ = imp.box_.borrow();
        let mut scales = scales.borrow_mut();

        // Remove scales?
        scales.retain(|stream_idx, scale| {
            let keep = streams.contains_key(stream_idx);
            if !keep {
                box_.remove(scale);
            }
            keep
        });

        for (stream_idx, stream) in streams {
            // Add or get sink from hashmap
            let scale = scales.entry(*stream_idx).or_default();

            // Append sink widget
            if scale.parent().is_none() {
                box_.append(scale);
            }

            scale.update(&stream.data);
        }
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

impl Default for MixerWindow {
    fn default() -> Self {
        Self::new()
    }
}
