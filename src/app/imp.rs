use std::cell::{Cell, OnceCell, RefCell};
use std::time::Duration;

use gdk::prelude::ApplicationExtManual;
use gdk::subclass::prelude::{ApplicationImpl, ApplicationImplExt};
use glib::clone;
use glib::subclass::object::{ObjectImpl, ObjectImplExt};
use glib::subclass::types::{ObjectSubclass, ObjectSubclassExt};
use gtk::gio;
use gtk::prelude::SettingsExt;
use gtk::subclass::prelude::GtkApplicationImpl;
use ksni::{Handle, TrayService};

use crate::constants::{MAX_NATURAL_VOL, SETTINGS_MOUSE_WHEEL_STEP};
use crate::pulse::Pulse;
use crate::ui::{
    mixer_window::MixerWindow,
    tray::{TrayMessage, VolctlTray},
};

pub struct Application {
    pub(super) _first_volume_update: RefCell<bool>,
    pub(super) hold_guard: RefCell<Option<gio::ApplicationHoldGuard>>,
    pub(super) mixer_window: OnceCell<MixerWindow>,
    pub(super) pulse: RefCell<Pulse>,
    pub(super) settings: OnceCell<gio::Settings>,
    pub(super) tray_handle: RefCell<Option<Handle<VolctlTray>>>,

    // Previous values
    pub(super) volume: Cell<u32>,
    pub(super) muted: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for Application {
    const NAME: &'static str = "VolctlApplication";
    type Type = super::Application;
    type ParentType = gtk::Application;
}

impl ObjectImpl for Application {
    fn constructed(&self) {
        self.parent_constructed();
    }
}

impl ApplicationImpl for Application {
    fn activate(&self) {
        self.parent_activate();
        let app = self.obj();

        // Prevent GTK main loop from exiting without window.
        *self.hold_guard.borrow_mut() = Some(app.hold());

        self.settings
            .get()
            .unwrap()
            .connect_changed(Some("changed::mouse-wheel-step"), |_, _| {
                println!("Settings changed");
            });

        // Connect to PulseAudio
        let mut pulse = self.pulse.borrow_mut();
        pulse.connect();

        // Periodically update widgets from PulseAudio
        glib::timeout_add_local(
            Duration::from_millis(1000 / 30),
            clone!(
                #[strong]
                app,
                move || {
                    app.update();
                    glib::ControlFlow::Continue
                }
            ),
        );

        // https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html#channels
        let (tx, rx) = async_channel::bounded::<TrayMessage>(1);

        // Start tray service
        let tray_service = TrayService::new(VolctlTray {
            tx,
            volume: 0,
            muted: false,
        });
        *self.tray_handle.borrow_mut() = Some(tray_service.handle());
        tray_service.spawn();

        // Listen for messages from the tray thread
        let settings_clone = self.settings.get().unwrap().clone();
        glib::spawn_future_local(clone!(
            #[weak]
            app,
            async move {
                while let Ok(msg) = rx.recv().await {
                    match msg {
                        TrayMessage::About => app.show_about(),
                        TrayMessage::Activate(x, y) => app.toggle_mixer_popup(x, y),
                        TrayMessage::ExternalMixer => app.external_mixer(),
                        TrayMessage::Mute => app.toggle_muted_active_sink_volume(),
                        TrayMessage::Preferences => app.show_prefs(),
                        TrayMessage::Scroll(delta) => {
                            let step = settings_clone.int(SETTINGS_MOUSE_WHEEL_STEP);
                            app.change_active_sink_volume(
                                (-delta as f32 * (step as f32 / 100.0) * MAX_NATURAL_VOL as f32)
                                    .round() as i32,
                            );
                        }
                        TrayMessage::Quit => app.request_quit(),
                    }
                }
            }
        ));

        let mixer_window = self.mixer_window.get().unwrap();
        mixer_window.build_ui();
    }
}

impl GtkApplicationImpl for Application {}

impl Default for Application {
    fn default() -> Self {
        Self {
            _first_volume_update: RefCell::from(false),
            hold_guard: RefCell::from(None),
            mixer_window: OnceCell::from(MixerWindow::new()),
            pulse: RefCell::from(Pulse::new()),
            settings: OnceCell::from(gio::Settings::with_path("apps.volctl", "/apps/volctl/")),
            tray_handle: RefCell::from(None),
            volume: Cell::new(0),
            muted: Cell::new(false),
        }
    }
}
