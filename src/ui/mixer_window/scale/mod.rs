use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::{ButtonExt, RangeExt, ToggleButtonExt, WidgetExt};
use gtk::{Image, Orientation};

use crate::constants::MAX_NATURAL_VOL;
use crate::pulse::MeterData;

mod imp;

glib::wrapper! {
  pub struct VolumeScale(ObjectSubclass<imp::VolumeScale>)
      @extends gtk::Box, gtk::Widget,
      @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl VolumeScale {
    pub fn new() -> Self {
        let box_: Self = glib::Object::builder()
            .property("orientation", Orientation::Vertical)
            .build();
        box_
    }

    pub fn update(&self, data: &MeterData) {
        let imp = self.imp();
        let mut scale_data = imp.data.borrow_mut();
        let scale = &imp.scale;
        let mute_btn = &imp.mute_btn;

        if data.icon != scale_data.icon {
            scale_data.icon = data.icon.to_owned();
            let icon = Image::from_icon_name(&scale_data.icon);
            mute_btn.set_child(Some(&icon));
        }

        if data.name != scale_data.name {
            scale_data.name = data.name.clone();
        }

        if data.description != scale_data.description {
            scale_data.description = data.description.clone();
            scale.set_tooltip_text(Some(&scale_data.description));
            mute_btn.set_tooltip_text(Some(&scale_data.description));
        }

        if data.volume != scale_data.volume {
            scale_data.volume = data.volume;
            scale.set_value(data.volume.avg().0 as f64);
        }

        if data.muted != scale_data.muted {
            scale_data.muted = data.muted;
            scale.set_sensitive(!data.muted);
            mute_btn.set_active(data.muted);
        }
    }

    fn format_scale_value(value: f64) -> String {
        format!("{:.0}", (value / MAX_NATURAL_VOL as f64) * 100.0)
    }
}
