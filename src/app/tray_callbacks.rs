use std::cmp::Ordering;

use gdk::prelude::SettingsExt;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::GtkWindowExt;
use libpulse::volume::Volume;

use crate::constants::{
    DEFAULT_MIXER_CMD, MAX_NATURAL_VOL, MAX_SCALE_VOL, SETTINGS_ALLOW_EXTRA_VOLUME,
    SETTINGS_MIXER_COMMAND,
};
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
                Ok(DisplayType::X11) => imp.x11_context,
                Ok(DisplayType::Wayland) | Err(_) => None,
            };
            let mixer_window = imp.mixer_window.clone();
            let window = MixerWindow::new(
                self,
                imp.pulse.clone(),
                imp.settings.clone(),
                x11_context,
                mixer_window,
            );
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
                    let extra_volume = imp.settings.boolean(SETTINGS_ALLOW_EXTRA_VOLUME);
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
    pub fn show_about() {
        let about_dialog = crate::ui::about_dialog::new();
        about_dialog.present();
    }

    /// Show preferences dialog
    pub fn show_prefs() {
        let prefs_window = PreferencesWindow::default();
        prefs_window.present();
    }

    /// Open external mixer program
    pub fn external_mixer(&self) {
        let imp = self.imp();
        let cmd_str = imp.settings.string(SETTINGS_MIXER_COMMAND);

        // Parse command: use default if empty, otherwise shell-split
        let mut args: Vec<String> = if cmd_str.is_empty() {
            vec![DEFAULT_MIXER_CMD.into()]
        } else {
            shlex::Shlex::new(&cmd_str).collect()
        };

        if args.is_empty() {
            tracing::warn!("Empty mixer command after parsing");
            return;
        }

        // Check if previous process is still running
        let mut mixer_child = imp.mixer_child.borrow_mut();
        if let Some(child) = mixer_child.as_mut() {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    // Process already exited, clear and spawn new one
                    *mixer_child = None;
                }
                Ok(None) => {
                    // Still running: do nothing (matches Python's poll() is None check)
                    return;
                }
                Err(e) => {
                    tracing::error!(error = %e, "Error checking mixer process status");
                }
            }
        }

        // Spawn new process
        let cmd = args.remove(0);
        match std::process::Command::new(&cmd).args(&args).spawn() {
            Ok(child) => {
                *mixer_child = Some(child);
            }
            Err(e) => {
                tracing::error!(cmd = %cmd, error = %e, "Failed to launch mixer");
            }
        }
    }

    /// Request volctl to quit
    pub fn request_quit(&self) {
        let imp = self.imp();

        // Cancel the periodic update timer
        if let Some(source_id) = imp.update_timer.borrow_mut().take() {
            source_id.remove();
        }

        // Close mixer window
        if let Some(win) = imp.mixer_window.take() {
            win.close();
        }

        // Destroy OSD (cancels timers, destroys surface/window)
        if let Some(osd_controller) = imp.osd_controller.borrow().as_ref() {
            osd_controller.destroy();
        }

        // Discard application hold guard which will make the GTK main loop terminate
        *imp.hold_guard.borrow_mut() = None;
    }
}
