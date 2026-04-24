use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use glib::subclass::object::ObjectImpl;
use glib::subclass::types::ObjectSubclass;
use gtk::gio;
use gtk::subclass::prelude::GtkApplicationImpl;
use ksni::blocking::Handle;

use crate::pulse::Pulse;
use crate::ui::osd::OsdController;
use crate::ui::{mixer_window::MixerWindow, tray::VolctlTray};

mod activate;

pub struct Application {
    pub(super) hold_guard: RefCell<Option<gio::ApplicationHoldGuard>>,
    pub(super) mixer_window: RefCell<Option<MixerWindow>>,
    pub(super) osd_controller: OnceCell<OsdController>,
    pub(super) pulse: Rc<RefCell<Pulse>>,
    pub(super) settings: OnceCell<gio::Settings>,
    pub(super) tray_handle: RefCell<Option<Handle<VolctlTray>>>,

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
        let osd_controller = OsdController::new(&settings);

        Self {
            hold_guard: RefCell::from(None),
            mixer_window: RefCell::from(None),
            osd_controller: OnceCell::from(osd_controller),
            pulse: Rc::from(RefCell::from(pulse_instance)),
            settings: OnceCell::from(settings),
            tray_handle: RefCell::from(None),
            volume: Cell::new(0),
            muted: Cell::new(false),
        }
    }
}
