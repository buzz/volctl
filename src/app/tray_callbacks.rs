use std::cmp::Ordering;

use gdk::prelude::SettingsExt;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::GtkWindowExt;
use libpulse::volume::Volume;

use crate::constants::{MAX_NATURAL_VOL, MAX_SCALE_VOL, SETTINGS_ALLOW_EXTRA_VOLUME};
use crate::pulse::StreamType;
use crate::ui::mixer_window::MixerWindow;
use crate::ui::prefs_window::PreferencesWindow;
use crate::ui::utils::{DisplayType, get_display_type};

use super::Application;

impl Application {
    /// Show/hide mixer popup
    pub fn toggle_mixer_popup(&self, x: i32, y: i32) {
        let imp = self.imp();

        if let Some(window) = imp.mixer_window.take() {
            window.close();
        } else {
            let x11_context = match get_display_type() {
                DisplayType::X11 => imp.x11_context,
                _ => None,
            };
            let window = MixerWindow::new(imp.pulse.clone(), x11_context);
            window.move_(x, y);

            // Populate with current PulseAudio data before showing
            let pulse = imp.pulse.borrow();
            window.update_sinks(pulse.get_sinks());
            window.update_sink_inputs(pulse.get_sink_inputs());
            drop(pulse);

            window.present();
            *imp.mixer_window.borrow_mut() = Some(window);
        }
    }

    /// Change active sink volume
    pub fn change_active_sink_volume(&self, amount: i32) {
        let imp = self.imp();
        let pulse = imp.pulse.borrow();

        if let Some(active_sink) = pulse.get_sinks().get(&pulse.active_sink) {
            let mut volumes = active_sink.data.volume;

            match amount.cmp(&0) {
                Ordering::Greater => {
                    let extra_volume = imp
                        .settings
                        .get()
                        .unwrap()
                        .boolean(SETTINGS_ALLOW_EXTRA_VOLUME);
                    let limit = match extra_volume {
                        true => MAX_SCALE_VOL,
                        false => MAX_NATURAL_VOL,
                    };
                    volumes.inc_clamp(Volume(amount as u32), Volume(limit));
                }
                Ordering::Less => {
                    volumes.decrease(Volume(amount.unsigned_abs()));
                }
                Ordering::Equal => {}
            }

            pulse.set_volume(StreamType::Sink, pulse.active_sink, volumes);
        }
    }

    /// Toggle muted on active sink volume
    pub fn toggle_muted_active_sink_volume(&self) {
        let pulse = self.imp().pulse.borrow();

        if let Some(active_sink) = pulse.get_sinks().get(&pulse.active_sink) {
            pulse.set_muted(StreamType::Sink, pulse.active_sink, !active_sink.data.muted);
        }
    }

    /// Show about dialog
    pub fn show_about(&self) {
        let about_dialog = crate::ui::about_dialog::new();
        about_dialog.present();
    }

    /// Show preferences dialog
    pub fn show_prefs(&self) {
        let prefs_window = PreferencesWindow::default();
        prefs_window.present();
    }

    /// Open external mixer program
    pub fn external_mixer(&self) {
        // TODO: open mixer
    }

    /// Request volctl to quit
    pub fn request_quit(&self) {
        let imp = self.imp();

        // Close mixer window
        if let Some(win) = imp.mixer_window.take() {
            win.close();
        }

        // Destroy OSD (cancels timers, destroys surface/window)
        if let Some(osd_controller) = imp.osd_controller.get() {
            osd_controller.destroy();
        }

        // Discard application hold guard which will make the GTK main loop terminate
        *imp.hold_guard.borrow_mut() = None;
    }
}
