use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::SettingsExt;
use gtk_layer_shell::{Layer, LayerShell};

use crate::constants::{SETTINGS_MIXER_POSITION, SETTINGS_USE_LAYER_SHELL};
use crate::ui::utils::{Position, apply_layer_shell_position};

use super::MixerWindow;

const MIXER_MARGIN: i32 = 32;

// Wayland
impl MixerWindow {
    pub fn move_wayland(&self, _x: i32, _y: i32) {
        let settings = self.imp().settings.get().expect("settings not set");

        let use_layer_shell = settings.boolean(SETTINGS_USE_LAYER_SHELL);
        if !use_layer_shell {
            // Without layer shell, the window is managed by the compositor.
            return;
        }

        self.init_layer_shell();
        self.set_layer(Layer::Overlay);
        self.set_exclusive_zone(0);

        let position = Position::try_from(settings.enum_(SETTINGS_MIXER_POSITION))
            .expect("invalid mixer-position value");
        apply_layer_shell_position(self, position, MIXER_MARGIN);
    }
}
