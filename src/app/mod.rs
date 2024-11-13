use glib::subclass::types::ObjectSubclassIsExt;
use gtk::{gio, prelude::WidgetExt};

use crate::constants::APP_ID;

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

    /// Process updates from PulseAudio
    fn update(&self) {
        let imp = self.imp();
        let mut pulse = imp.pulse.borrow_mut();

        if pulse.update() {
            // Update tray icon
            if let Some(active_sink) = pulse.sinks.get(&pulse.active_sink) {
                let new_volume = active_sink.data.volume.avg().0;
                let new_muted = active_sink.data.muted;

                // Only send update if values changed
                if new_volume != imp.volume.get() || new_muted != imp.muted.get() {
                    if let Some(tray_handle) = imp.tray_handle.borrow().as_ref() {
                        tray_handle.update(|tray| {
                            tray.volume = new_volume;
                            tray.muted = new_muted;
                        });
                    }

                    // Remember new values
                    imp.volume.set(new_volume);
                    imp.muted.set(new_muted);
                }
            }

            // Update mixer window if it's visible
            if let Some(mixer_window) = imp.mixer_window.get() {
                if mixer_window.get_visible() {
                    mixer_window.update_sinks(&pulse.sinks);
                    mixer_window.update_sink_inputs(&pulse.sink_inputs);
                    // TODO: peaks
                }
            }
        };
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}
