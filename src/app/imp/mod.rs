use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use glib::subclass::object::ObjectImpl;
use glib::subclass::types::ObjectSubclass;
use gtk::gio;
use gtk::subclass::prelude::GtkApplicationImpl;
use ksni::Handle;

use crate::pulse::Pulse;
use crate::ui::{mixer_window::MixerWindow, tray::VolctlTray};

mod activate;

pub struct Application {
    pub(super) _first_volume_update: RefCell<bool>,
    pub(super) hold_guard: RefCell<Option<gio::ApplicationHoldGuard>>,
    pub(super) mixer_window: OnceCell<MixerWindow>,
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
        Self {
            _first_volume_update: RefCell::from(false),
            hold_guard: RefCell::from(None),
            mixer_window: OnceCell::new(),
            pulse: Rc::from(RefCell::from(Pulse::new())),
            settings: OnceCell::from(gio::Settings::with_path("apps.volctl", "/apps/volctl/")),
            tray_handle: RefCell::from(None),
            volume: Cell::new(0),
            muted: Cell::new(false),
        }
    }
}
