use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::SettingsExt;
use gtk::{gio, prelude::WidgetExt};

use crate::constants::{APP_ID, SETTINGS_VU_ENABLED};

mod imp;
mod tray_callbacks;

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

    /// Process updates from `PulseAudio`
    fn update(&self) {
        let imp = self.imp();

        // Process incoming PulseAudio messages (needs mutable borrow).
        let received = {
            let mut pulse = imp.pulse.borrow_mut();
            pulse.update()
        };

        // Only update tray/OSD when actual PulseAudio data changes arrive.
        // Peak decay happens every frame inside pulse.update() regardless.
        if received {
            let (active_sink_volume, active_sink_muted) = {
                let pulse = imp.pulse.borrow();
                if let Some(active_sink) = pulse.get_sinks().get(&pulse.active_sink) {
                    (active_sink.data.volume.avg().0, active_sink.data.muted)
                } else {
                    return;
                }
            };

            // Only send update if values changed
            if active_sink_volume != imp.volume.get() || active_sink_muted != imp.muted.get() {
                // Update tray icon
                if let Some(tray_handle) = imp.tray_handle.borrow().as_ref() {
                    tray_handle.update(|tray| {
                        tray.volume = active_sink_volume;
                        tray.muted = active_sink_muted;
                    });
                }

                // Update OSD
                if let Some(osd_controller) = imp.osd_controller.borrow().as_ref() {
                    osd_controller.update(active_sink_volume, active_sink_muted);
                }

                // Remember new values
                imp.volume.set(active_sink_volume);
                imp.muted.set(active_sink_muted);
            }
        }

        // Update mixer window every frame: peaks decay continuously even
        // when no PulseAudio messages arrive, so the UI must refresh every frame.
        // Use an immutable borrow so value_changed handlers (which also borrow
        // pulse immutably) don't conflict with a mutable borrow.
        let mixer_visible = if let Some(mixer_window) = imp.mixer_window.borrow().as_ref()
            && mixer_window.get_visible()
        {
            let pulse = imp.pulse.borrow();
            mixer_window.update_sinks(pulse.get_sinks());
            mixer_window.update_sink_inputs(pulse.get_sink_inputs());
            true
        } else {
            false
        };

        // Enable/disable peak monitoring based on mixer visibility and VU setting.
        // Must run every frame so VU enables promptly when the mixer opens.
        let vu_enabled = imp.settings.boolean(SETTINGS_VU_ENABLED) && mixer_visible;
        if imp.pulse.borrow().is_vu_enabled() != vu_enabled {
            imp.pulse.borrow_mut().set_vu_enabled(vu_enabled);
        }
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}
