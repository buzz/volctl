use glib::subclass::types::ObjectSubclassIsExt;
use gtk::gio;

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
