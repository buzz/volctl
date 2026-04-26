use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use glib::subclass::object::ObjectImpl;
use glib::subclass::types::ObjectSubclass;
use gtk::gio;
use gtk::subclass::prelude::GtkApplicationImpl;
use ksni::blocking::Handle;

use crate::pulse::Pulse;
use crate::ui::osd::OsdController;
use crate::ui::utils::{DisplayType, get_display_type};
use crate::ui::x11::X11Context;
use crate::ui::{mixer_window::MixerWindow, tray::VolctlTray};

mod activate;

pub struct Application {
    pub(super) hold_guard: RefCell<Option<gio::ApplicationHoldGuard>>,
    pub(super) mixer_window: RefCell<Option<MixerWindow>>,
    pub(super) osd_controller: OnceCell<OsdController>,
    pub(super) pulse: Rc<RefCell<Pulse>>,
    pub(super) settings: OnceCell<gio::Settings>,
    pub(super) tray_handle: RefCell<Option<Handle<VolctlTray>>>,
    pub(super) x11_context: Option<X11Context>,

    // Previous values (tray icon)
    pub(super) volume: Cell<u32>,
    pub(super) muted: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for Application {
    const NAME: &'static str = "VolctlApplication";
    type Type = super::Application;
    type ParentType = gtk::Application;
}

impl ObjectImpl for Application {}

impl GtkApplicationImpl for Application {}

impl Default for Application {
    fn default() -> Self {
        let pulse_instance = Pulse::new().expect("Failed to create PulseAudio controller");
        let settings = gio::Settings::with_path("apps.volctl", "/apps/volctl/");

        // X11Context is Copy, so passing Some(ctx) does not consume the original
        let x11_context = match get_display_type() {
            DisplayType::X11 => Some(X11Context::default()),
            _ => None,
        };

        let osd_controller = OsdController::new(&settings, x11_context);

        Self {
            hold_guard: RefCell::from(None),
            mixer_window: RefCell::from(None),
            osd_controller: OnceCell::from(osd_controller),
            pulse: Rc::from(RefCell::from(pulse_instance)),
            settings: OnceCell::from(settings),
            tray_handle: RefCell::from(None),
            x11_context,
            volume: Cell::new(0),
            muted: Cell::new(false),
        }
    }
}
