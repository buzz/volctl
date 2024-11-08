use std::cell::RefCell;
use std::rc::Rc;

use glib::subclass::object::ObjectImplExt;
use glib::subclass::types::ObjectSubclassExt;
use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
use gtk::prelude::{BoxExt, ScaleExt, WidgetExt};
use gtk::subclass::{box_::BoxImpl, widget::WidgetImpl};
use gtk::{Adjustment, Orientation, PositionType, Scale, ToggleButton};

use crate::constants::MAX_NATURAL_VOL;
use crate::pulse::MeterData;

#[derive(Debug)]
pub struct VolumeScale {
    pub(super) scale: Scale,
    pub(super) mute_btn: ToggleButton,
    pub(super) data: Rc<RefCell<MeterData>>,
}

#[glib::object_subclass]
impl ObjectSubclass for VolumeScale {
    const NAME: &'static str = "VolctlVolumeScale";
    type Type = super::VolumeScale;
    type ParentType = gtk::Box;
}

impl ObjectImpl for VolumeScale {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        self.scale.set_size_request(24, 128);
        self.scale
            .set_format_value_func(|_, value| super::VolumeScale::format_scale_value(value));

        obj.append(&self.scale);
        obj.append(&self.mute_btn);
    }
}

impl WidgetImpl for VolumeScale {}

impl BoxImpl for VolumeScale {}

impl Default for VolumeScale {
    fn default() -> Self {
        Self {
            scale: Scale::builder()
                .orientation(Orientation::Vertical)
                .adjustment(
                    &Adjustment::builder()
                        .step_increment(10.0)
                        .lower(0.0)
                        .upper(MAX_NATURAL_VOL as f64)
                        .build(),
                )
                .inverted(true)
                .draw_value(true)
                .value_pos(PositionType::Bottom)
                .build(),
            mute_btn: ToggleButton::builder().margin_bottom(6).build(),
            data: Rc::from(RefCell::from(MeterData::default())),
        }
    }
}
