use gtk::gio;
use gtk::prelude::GtkWindowExt;
use std::rc::Rc;

use crate::ui::osd::controller::OsdStateController;

pub struct WaylandSurface {
    window: gtk::Window,
}

impl WaylandSurface {
    pub fn new(_settings: &gio::Settings, _controller: Rc<OsdStateController>) -> Self {
        let window: gtk::Window = glib::Object::builder()
            .property("decorated", false)
            .property("resizable", false)
            .build();

        Self { window }
    }
}

impl super::SurfaceBackend for WaylandSurface {
    fn show(&self) {
        self.window.present();
    }

    fn update_position(&self, _position: &str) {
        // TODO: Implement Wayland positioning using layer-shell anchors
    }

    fn update_scale(&self, _scale: f64) {
        // TODO: Implement Wayland scale updates
    }

    fn begin_fade_out(&self, _opacity: f64) {
        // TODO: Apply opacity for Wayland fade-out
    }

    fn is_composited(&self) -> bool {
        // Wayland always has a compositor
        true
    }

    fn destroy(&self) {
        self.window.destroy();
    }
}
