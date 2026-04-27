use std::cell::RefCell;
use std::rc::Rc;

use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::{ButtonExt, ObjectExt, RangeExt, ToggleButtonExt, WidgetExt};
use gtk::{Image, Orientation};
use libpulse::volume::{ChannelVolumes, Volume};

use crate::constants::MAX_NATURAL_VOL;
use crate::pulse::{MeterData, Pulse};

mod imp;

glib::wrapper! {
  pub struct VolumeScale(ObjectSubclass<imp::VolumeScale>)
      @extends gtk::Box, gtk::Widget,
      @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl VolumeScale {
    pub fn new(pulse: Rc<RefCell<Pulse>>) -> Self {
        let obj: Self = glib::Object::builder()
            .property("orientation", Orientation::Vertical)
            .build();

        let imp = obj.imp();
        imp.pulse.set(pulse.clone()).ok();

        // Clone the Rc reference outside the closure so the imp() borrow is released
        let data: Rc<RefCell<MeterData>> = Rc::clone(&imp.data);
        let scale = imp.scale.clone();

        // Connect slider signal: store handler ID so update() can block it
        {
            let pulse = pulse.clone();
            let data = Rc::clone(&data);
            let handler_id = scale.connect_value_changed(move |scale| {
                let d = data.borrow();
                let index = d.index;
                let stream_type = d.t;
                let ch_count = d.volume.len();
                drop(d);

                // Guard against default MeterData (ch_count == 0) before first update()
                if ch_count == 0 {
                    return;
                }

                let value = scale.value() as u32;
                let mut volumes = ChannelVolumes::default();
                volumes.set_len(ch_count);
                volumes.set(ch_count, Volume(value));

                let p = pulse.borrow();
                p.set_volume(stream_type, index, volumes);
            });
            imp.value_changed_handler.set(handler_id).ok();
        }

        // Connect mute button signal: store handler ID so update() can block it
        {
            let pulse = pulse.clone();
            let data = Rc::clone(&data);
            let mute_btn = imp.mute_btn.clone();
            let handler_id = mute_btn.connect_toggled(move |btn| {
                let d = data.borrow();
                let index = d.index;
                let stream_type = d.t;
                drop(d);

                let muted = btn.is_active();
                let p = pulse.borrow();
                p.set_muted(stream_type, index, muted);
            });
            imp.toggled_handler.set(handler_id).ok();
        }

        obj
    }

    pub fn update(&self, data: &MeterData) {
        let imp = self.imp();

        // Collect all values inside a single borrow, then drop it before GTK setters
        let (new_volume, new_muted, new_icon, new_tooltip, needs_icon) = {
            let mut scale_data = imp.data.borrow_mut();
            // Always copy identity fields (type + index) so signal handlers
            // target the correct stream: they default to StreamType::Sink/0.
            scale_data.t = data.t;
            scale_data.index = data.index;

            let icon_changed = data.icon != scale_data.icon;
            if icon_changed {
                scale_data.icon.clone_from(&data.icon);
            }
            if data.name != scale_data.name {
                scale_data.name.clone_from(&data.name);
            }
            let tooltip_changed = data.description != scale_data.description;
            if tooltip_changed {
                scale_data.description.clone_from(&data.description);
            }
            let volume_changed = data.volume != scale_data.volume;
            if volume_changed {
                scale_data.volume = data.volume;
            }
            let mute_changed = data.muted != scale_data.muted;
            if mute_changed {
                scale_data.muted = data.muted;
            }

            (
                if volume_changed {
                    Some(data.volume.avg().0 as f64)
                } else {
                    None
                },
                if mute_changed { Some(data.muted) } else { None },
                if icon_changed {
                    Some(scale_data.icon.clone())
                } else {
                    None
                },
                if tooltip_changed {
                    Some(scale_data.description.clone())
                } else {
                    None
                },
                icon_changed,
            )
        }; // borrow_mut() drops here: RefCell is now free

        // Block GTK signal handlers before calling setters that may fire signals
        if let Some(id) = imp.value_changed_handler.get() {
            imp.scale.block_signal(id);
        }
        if let Some(id) = imp.toggled_handler.get() {
            imp.mute_btn.block_signal(id);
        }

        if needs_icon {
            let icon = Image::from_icon_name(&new_icon.unwrap());
            imp.mute_btn.set_child(Some(&icon));
        }

        if let Some(tooltip) = new_tooltip {
            imp.scale.set_tooltip_text(Some(&tooltip));
            imp.mute_btn.set_tooltip_text(Some(&tooltip));
        }

        if let Some(value) = new_volume {
            imp.scale.set_value(value);
        }

        if let Some(muted) = new_muted {
            imp.scale.set_sensitive(!muted);
            imp.mute_btn.set_active(muted);
        }

        // Unblock handlers
        if let Some(id) = imp.value_changed_handler.get() {
            imp.scale.unblock_signal(id);
        }
        if let Some(id) = imp.toggled_handler.get() {
            imp.mute_btn.unblock_signal(id);
        }
    }

    fn format_scale_value(value: f64) -> String {
        format!("{:.0}", (value / MAX_NATURAL_VOL as f64) * 100.0)
    }
}
