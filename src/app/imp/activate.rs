use std::time::Duration;

use gdk::prelude::ApplicationExtManual;
use gdk::subclass::prelude::{ApplicationImpl, ApplicationImplExt};
use glib::clone;
use glib::subclass::types::ObjectSubclassExt;
use gtk::prelude::SettingsExt;
use ksni::TrayService;

use crate::constants::{MAX_NATURAL_VOL, SETTINGS_MOUSE_WHEEL_STEP};
use crate::ui::{
    mixer_window::MixerWindow,
    tray::{TrayMessage, VolctlTray},
};

use super::Application;

impl ApplicationImpl for Application {
    fn activate(&self) {
        self.parent_activate();

        // Prevent GTK main loop from exiting without window.
        *self.hold_guard.borrow_mut() = Some(self.obj().hold());

        self.init_tray();
        self.create_mixer_window();
        self.init_pulse();
    }
}

impl Application {
    fn create_mixer_window(&self) {
        self.mixer_window
            .set(MixerWindow::new())
            .expect("Failed to set mixer window.");
    }

    fn init_pulse(&self) {
        // Connect to PulseAudio
        self.pulse.borrow_mut().connect();

        // Periodically update widgets from PulseAudio
        let app = self.obj();
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
    }

    fn init_tray(&self) {
        // Tray service lives in another thread, so we communicate via a channel.
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
        let settings_clone = self
            .settings
            .get()
            .expect("Failed to get settings.")
            .clone();
        let app = self.obj();
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
                                // scale step (in %) to MAX_NATURAL_VOL
                                (-delta as f32 * (step as f32 / 100.0) * MAX_NATURAL_VOL as f32)
                                    .round() as i32,
                            );
                        }
                        TrayMessage::Quit => app.request_quit(),
                    }
                }
            }
        ));
    }
}
