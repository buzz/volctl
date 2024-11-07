use std::cell::{Cell, OnceCell, RefCell};
use std::time::Duration;

use gdk::prelude::ApplicationExtManual;
use gdk::subclass::prelude::{ApplicationImpl, ApplicationImplExt};
use glib::clone;
use glib::subclass::object::{ObjectImpl, ObjectImplExt};
use glib::subclass::types::{ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt};
use gtk::gio;
use gtk::prelude::{GtkWindowExt, SettingsExt, WidgetExt};
use gtk::subclass::prelude::GtkApplicationImpl;
use ksni::{Handle, TrayService};
use libpulse::volume::Volume;

use crate::constants::{
    APP_ID, MAX_NATURAL_VOL, MAX_SCALE_VOL, SETTINGS_ALLOW_EXTRA_VOLUME, SETTINGS_MOUSE_WHEEL_STEP,
};
use crate::pulse::StreamType;

use super::pulse::Pulse;
use super::ui::{
    mixer_window::MixerWindow,
    tray::{TrayMessage, VolctlTray},
};

mod imp {
    use super::*;

    pub struct Application {
        pub(super) first_volume_update: RefCell<bool>,
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

            self.settings.get().unwrap().connect_changed(
                Some("changed::mouse-wheel-step"),
                |_, _| {
                    println!("Settings changed");
                },
            );

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
            let (tx, rx) = async_channel::bounded(1);

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
                            TrayMessage::Activate(x, y) => app.toggle_mixer(x, y),
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
                first_volume_update: RefCell::from(false),
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
}

glib::wrapper! {
  pub struct Application(ObjectSubclass<imp::Application>)
      @extends gio::Application, gtk::Application,
      @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .build()
    }

    /// Show/hide mixer window.
    pub fn toggle_mixer(&self, x: i32, y: i32) {
        let window = self.imp().mixer_window.get().unwrap();
        if window.get_visible() {
            window.set_visible(false);
        } else {
            window.move_(x, y);
            window.present();
        }
    }

    /// Change active sink volume.
    fn change_active_sink_volume(&self, amount: i32) {
        let imp = self.imp();
        let pulse = imp.pulse.borrow();

        if let Some(active_sink) = pulse.sinks.get(&pulse.active_sink) {
            let mut volumes = active_sink.data.volume.clone();

            if amount > 0 {
                let extra_volume = imp
                    .settings
                    .get()
                    .unwrap()
                    .boolean(SETTINGS_ALLOW_EXTRA_VOLUME);
                let limit = if extra_volume {
                    MAX_SCALE_VOL
                } else {
                    MAX_NATURAL_VOL
                };
                volumes.inc_clamp(Volume(amount as u32), Volume(limit));
            } else if amount < 0 {
                volumes.decrease(Volume(amount.abs() as u32));
            }

            pulse.set_volume(StreamType::Sink, pulse.active_sink, volumes);
        }
    }

    /// Toggle muted on active sink volume.
    fn toggle_muted_active_sink_volume(&self) {
        let pulse = self.imp().pulse.borrow();

        if let Some(active_sink) = pulse.sinks.get(&pulse.active_sink) {
            pulse.set_muted(StreamType::Sink, pulse.active_sink, !active_sink.data.muted);
        }
    }

    /// Show about dialog.
    fn show_about(&self) {}

    /// Show preferences dialog.
    fn show_prefs(&self) {}

    /// Open external mixer program.
    fn external_mixer(&self) {}

    /// Request volctl to quit.
    fn request_quit(&self) {
        let imp = self.imp();

        // Close pulse.
        imp.pulse.borrow_mut().cleanup();

        // Close mixer window.
        if let Some(win) = imp.mixer_window.get() {
            win.destroy();
        }

        // Discard application hold guard.
        *imp.hold_guard.borrow_mut() = None;
    }

    /// Process updates from PulseAudio.
    fn update(&self) {
        let imp = self.imp();
        let mut pulse = imp.pulse.borrow_mut();

        if pulse.update() {
            // Active sink
            if let Some(active_sink) = pulse.sinks.get(&pulse.active_sink) {
                let new_volume = active_sink.data.volume.avg().0;
                let new_muted = active_sink.data.muted;

                // Update tray icon?
                if new_volume != imp.volume.get() || new_muted != imp.muted.get() {
                    if let Some(tray_handle) = imp.tray_handle.borrow().as_ref() {
                        tray_handle.update(|tray| {
                            tray.volume = new_volume;
                            tray.muted = new_muted;
                        });
                    }
                }

                // Remember new values.
                imp.volume.set(new_volume);
                imp.muted.set(new_muted);
            };
        }
    }
}
