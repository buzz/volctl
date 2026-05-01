use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use glib::subclass::object::ObjectImplExt;
use glib::subclass::types::ObjectSubclassExt;
use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
use gtk::prelude::{BoxExt, ScaleExt};
use gtk::subclass::{box_::BoxImpl, widget::WidgetImpl};
use gtk::{Adjustment, Orientation, PositionType, Scale, ToggleButton};

use crate::pulse::{MeterData, Pulse};

pub struct VolumeScale {
    pub(super) scale: Scale,
    pub(super) mute_btn: ToggleButton,
    pub(super) data: Rc<RefCell<MeterData>>,
    /// Allow 150% extra volume.
    pub(super) allow_extra_volume: Cell<bool>,
    /// Set after construction. Used by signal handlers.
    pub(super) pulse: OnceCell<Rc<RefCell<Pulse>>>,
    /// GTK signal handler IDs so update() can block them during programmatic changes.
    pub(super) value_changed_handler: OnceCell<glib::SignalHandlerId>,
    pub(super) toggled_handler: OnceCell<glib::SignalHandlerId>,
    /// Settings reference for allow-extra-volume reactivity.
    pub(super) settings: OnceCell<gtk::gio::Settings>,
    /// Last displayed peak fill level (0.0..1.0). Used to skip redundant GTK setters.
    pub(super) last_displayed_peak: Cell<f64>,
    /// Whether VU meter display is enabled. Used to skip fill_level updates in update_peak().
    pub(super) vu_enabled: Cell<bool>,
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

        // Inset the `<marks>` container of vertical scales so snap-marks align
        // with the center of the knob (knob travel is shorter than full trough height).
        static SCALE_CSS_LOADED: std::sync::Once = std::sync::Once::new();
        SCALE_CSS_LOADED.call_once(|| {
            let provider = gtk::CssProvider::new();
            provider.load_from_string(
                r#"
                scale marks {
                    margin-top: 0;
                }
                scale value {
                    margin-top: 4px;
                }
                button.toggle {
                    padding: 0;
                    margin: 0;
                    border: none;
                    background: transparent;
                }
                button.toggle:hover {
                    background-color: transparent;
                    border-color: transparent;
                }
                button.toggle:checked {
                    background-color: transparent;
                    border-color: transparent;
                    opacity: 0.5;
                }
            "#,
            );
            if let Some(display) = gtk::gdk::Display::default() {
                gtk::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }
        });

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
        let adj = Adjustment::builder()
            .lower(0.0)
            .upper(1.0)
            .step_increment(0.01)
            .page_increment(0.1)
            .build();

        Self {
            scale: Scale::builder()
                .orientation(Orientation::Vertical)
                .adjustment(&adj)
                .round_digits(2)
                .digits(2)
                .inverted(true)
                .value_pos(PositionType::Bottom)
                .margin_top(4)
                .restrict_to_fill_level(false)
                .build(),
            mute_btn: ToggleButton::builder().build(),
            data: Rc::from(RefCell::from(MeterData::default())),
            allow_extra_volume: true.into(),
            pulse: OnceCell::new(),
            value_changed_handler: OnceCell::new(),
            toggled_handler: OnceCell::new(),
            settings: OnceCell::new(),
            last_displayed_peak: Cell::new(0.0),
            vu_enabled: Cell::new(false),
        }
    }
}
