use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use glib::subclass::object::ObjectImplExt;
use glib::subclass::types::ObjectSubclassExt;
use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
use gtk::prelude::{BoxExt, ScaleExt, WidgetExt};
use gtk::subclass::{box_::BoxImpl, widget::WidgetImpl};
use gtk::{Adjustment, Orientation, PositionType, Scale, ToggleButton};

use crate::constants::MAX_NATURAL_VOL;
use crate::pulse::{MeterData, Pulse};

pub struct VolumeScale {
    pub(super) scale: Scale,
    pub(super) mute_btn: ToggleButton,
    pub(super) data: Rc<RefCell<MeterData>>,
    /// Set after construction. Used by signal handlers.
    pub(super) pulse: OnceCell<Rc<RefCell<Pulse>>>,
    /// GTK signal handler IDs so update() can block them during programmatic changes.
    pub(super) value_changed_handler: OnceCell<glib::SignalHandlerId>,
    pub(super) toggled_handler: OnceCell<glib::SignalHandlerId>,
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
                .margin_top(4)
                .build(),
            mute_btn: ToggleButton::builder().build(),
            data: Rc::from(RefCell::from(MeterData::default())),
            pulse: OnceCell::new(),
            value_changed_handler: OnceCell::new(),
            toggled_handler: OnceCell::new(),
        }
    }
}
