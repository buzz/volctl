use std::cell::Cell;
use std::rc::Rc;

use gtk::gio;
use gtk::prelude::*;
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use tracing;

use crate::constants::{OSD_BASE_HEIGHT, OSD_BASE_WIDTH, OSD_SCREEN_MARGIN, SETTINGS_OSD_SCALE};
use crate::ui::osd::controller::OsdStateController;
use crate::ui::osd::widget::OsdWidget;

pub struct WaylandSurface {
    widget: OsdWidget,
    controller: Rc<OsdStateController>,
    scale: Cell<f64>,
}

impl WaylandSurface {
    pub fn new(settings: &gio::Settings, controller: Rc<OsdStateController>) -> Self {
        let scale = settings.int(SETTINGS_OSD_SCALE) as f64 / 100.0;

        let widget = OsdWidget::new(scale, true);
        let window = widget.window();

        // Layer Shell Setup
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_namespace(Some("volctl-volume-osd"));

        // Click-Through
        window.connect_realize(|win| {
            if let Some(surface) = win.surface() {
                // Create a region object that contains nothing
                let region = cairo::Region::create();
                surface.set_input_region(Some(&region));
            }
        });

        let surf = Self {
            widget,
            controller,
            scale: Cell::new(scale),
        };

        surf.update_size(scale);
        surf
    }

    fn update_size(&self, scale: f64) {
        let window = self.widget.window();
        let width = (OSD_BASE_WIDTH * scale) as i32;
        let height = (OSD_BASE_HEIGHT * scale) as i32;

        // In Layer Shell, set_default_size helps GTK calculate the initial allocation.
        // The compositor sizes the surface based on the anchor+margin+content-size negotiation.
        window.set_default_size(width, height);
    }
}

impl super::SurfaceBackend for WaylandSurface {
    fn show(&self) {
        let window = self.widget.window();

        let volume = self.controller.get_volume_normalized();
        let muted = self.controller.get_muted();
        let opacity = self.controller.get_opacity();
        self.widget.update_state(volume, muted, opacity);

        // Reset window opacity to fully visible before presenting
        window.set_opacity(1.0);
        window.present();
    }

    fn update_position(&self, position: &str) {
        let window = self.widget.window();
        let margin = OSD_SCREEN_MARGIN;

        // Reset all anchors and margins first
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);
        window.set_margin(Edge::Top, 0);
        window.set_margin(Edge::Bottom, 0);
        window.set_margin(Edge::Left, 0);
        window.set_margin(Edge::Right, 0);

        let parts: Vec<&str> = position.split('-').collect();
        if parts.len() != 2 {
            tracing::warn!(position = %position, "Invalid OSD position, falling back to center");
            return;
        }

        let v_pos = parts[0]; // top, center, bottom
        let h_pos = parts[1]; // left, center, right

        // Vertical Anchor & Margin
        match v_pos {
            "top" => {
                window.set_anchor(Edge::Top, true);
                window.set_margin(Edge::Top, margin);
            }
            "bottom" => {
                window.set_anchor(Edge::Bottom, true);
                window.set_margin(Edge::Bottom, margin);
            }
            "center" => {
                // To center vertically, leave both Top and Bottom anchors false.
                // Compositor handles centering.
            }
            _ => {}
        }

        // Horizontal Anchor & Margin
        match h_pos {
            "left" => {
                window.set_anchor(Edge::Left, true);
                window.set_margin(Edge::Left, margin);
            }
            "right" => {
                window.set_anchor(Edge::Right, true);
                window.set_margin(Edge::Right, margin);
            }
            "center" => {
                // To center horizontally, leave both Left and Right anchors false.
                // Compositor handles centering.
            }
            _ => {}
        }
    }

    fn update_scale(&self, scale: f64) {
        self.scale.set(scale);
        self.widget.update_scale(scale);
        self.update_size(scale);
    }

    fn begin_fade_out(&self, opacity: f64) {
        let window = self.widget.window();
        window.set_opacity(opacity);

        if opacity <= 0.0 {
            window.set_visible(false);
        }
    }

    fn is_composited(&self) -> bool {
        true
    }

    fn destroy(&self) {
        let window = self.widget.window();
        window.set_visible(false);
        window.destroy();
    }
}
