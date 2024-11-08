use gtk_layer_shell::{Edge, Layer, LayerShell};

use super::MixerWindow;

// Wayland
impl MixerWindow {
    pub fn move_wayland(&self, _x: i32, _y: i32) {
        self.init_layer_shell();
        self.set_layer(Layer::Overlay);
        self.auto_exclusive_zone_enable();

        self.set_margin(Edge::Right, 32);
        self.set_margin(Edge::Top, 32);

        self.set_anchor(Edge::Left, false);
        self.set_anchor(Edge::Right, true);
        self.set_anchor(Edge::Top, true);
        self.set_anchor(Edge::Bottom, false);
    }
}
