use std::cell::{Cell, RefCell};
use std::process::Child;
use std::rc::Rc;

use glib::SourceId;
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
    pub(super) mixer_child: RefCell<Option<Child>>,
    pub(super) mixer_window: Rc<RefCell<Option<MixerWindow>>>,
    pub(super) osd_controller: RefCell<Option<OsdController>>,
    pub(super) pulse: Rc<RefCell<Pulse>>,
    pub(super) settings: gio::Settings,
    pub(super) tray_handle: RefCell<Option<Handle<VolctlTray>>>,
    pub(super) x11_context: Option<X11Context>,

    pub(super) display_type: DisplayType,
    // Timer for periodic PulseAudio updates
    pub(super) update_timer: RefCell<Option<SourceId>>,

    // Previous values (tray icon)
    pub(super) volume: Cell<u32>,
    pub(super) muted: Cell<bool>,
    pub(super) use_symbolic_icons: Cell<bool>,
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

        let (display_type, x11_context) = match get_display_type() {
            Ok(DisplayType::X11) => {
                let ctx = X11Context::new().expect("X11 context required on X11 display");
                (DisplayType::X11, Some(ctx))
            }
            Ok(DisplayType::Wayland) => (DisplayType::Wayland, None),
            Err(e) => {
                tracing::error!(error = %e, "Failed to detect display type, assuming Wayland");
                (DisplayType::Wayland, None)
            }
        };

        Self {
            hold_guard: RefCell::from(None),
            mixer_child: RefCell::from(None),
            mixer_window: Rc::new(RefCell::new(None)),
            osd_controller: RefCell::from(None),
            pulse: Rc::from(RefCell::from(pulse_instance)),
            settings,
            tray_handle: RefCell::from(None),
            update_timer: RefCell::from(None),
            x11_context,
            display_type,
            volume: Cell::new(0),
            muted: Cell::new(false),
            use_symbolic_icons: Cell::new(true),
        }
    }
}
