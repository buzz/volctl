use std::cmp::Ordering;

use gdk::prelude::SettingsExt;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::{GtkWindowExt, WidgetExt};
use libpulse::volume::Volume;

use crate::constants::{MAX_NATURAL_VOL, MAX_SCALE_VOL, SETTINGS_ALLOW_EXTRA_VOLUME};
use crate::pulse::StreamType;

use super::Application;

impl Application {
    /// Show/hide mixer popup.
    pub fn toggle_mixer_popup(&self, x: i32, y: i32) {
        if let Some(window) = self.imp().mixer_window.get() {
            if window.get_visible() {
                window.set_visible(false);
            } else {
                window.move_(x, y);
                window.present();
            }
        }
    }

    /// Change active sink volume.
    pub fn change_active_sink_volume(&self, amount: i32) {
        let imp = self.imp();
        let pulse = imp.pulse.borrow();

        if let Some(active_sink) = pulse.sinks.get(&pulse.active_sink) {
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
            };

            pulse.set_volume(StreamType::Sink, pulse.active_sink, volumes);
        }
    }

    /// Toggle muted on active sink volume.
    pub fn toggle_muted_active_sink_volume(&self) {
        let pulse = self.imp().pulse.borrow();

        if let Some(active_sink) = pulse.sinks.get(&pulse.active_sink) {
            pulse.set_muted(StreamType::Sink, pulse.active_sink, !active_sink.data.muted);
        }
    }

    /// Show about dialog.
    pub fn show_about(&self) {}

    /// Show preferences dialog.
    pub fn show_prefs(&self) {}

    /// Open external mixer program.
    pub fn external_mixer(&self) {}

    /// Request volctl to quit.
    pub fn request_quit(&self) {
        let imp = self.imp();

        // Close pulse.
        imp.pulse.borrow_mut().cleanup();

        // Close mixer window.
        if let Some(win) = imp.mixer_window.get() {
            win.destroy();
        }

        // Discard application hold guard which will make the GTK main loop terminate.
        *imp.hold_guard.borrow_mut() = None;
    }
}
