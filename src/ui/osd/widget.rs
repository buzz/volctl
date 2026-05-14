use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

use crate::constants::{OSD_BASE_HEIGHT, OSD_BASE_WIDTH};
use crate::ui::osd::render;
use crate::ui::osd::render::RenderState;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct OsdRenderWidget {
        pub render_state: RefCell<RenderState>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OsdRenderWidget {
        const NAME: &'static str = "OsdRenderWidget";
        type Type = super::OsdRenderWidget;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for OsdRenderWidget {}

    impl WidgetImpl for OsdRenderWidget {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let state = self.render_state.borrow();
            // Get PangoContext from widget (has proper font map)
            let pango_context = self.obj().pango_context();
            render::build_snapshot(&state, snapshot, &pango_context);
        }
    }

    impl OsdRenderWidget {
        pub fn get_render_state(&self) -> RenderState {
            self.render_state.borrow().clone()
        }

        pub fn set_render_state(&self, state: RenderState) {
            *self.render_state.borrow_mut() = state;
        }
    }
}

glib::wrapper! {
    pub struct OsdRenderWidget(ObjectSubclass<imp::OsdRenderWidget>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for OsdRenderWidget {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl OsdRenderWidget {
    pub fn new(scale: f64) -> Self {
        Self::new_with_composited(scale, true)
    }

    pub fn new_with_composited(scale: f64, composited: bool) -> Self {
        let widget: Self = glib::Object::new();
        let imp = widget.imp();
        imp.set_render_state(RenderState {
            volume: 0.0,
            muted: false,
            opacity: 1.0,
            scale,
            composited,
        });
        widget
    }

    pub fn update_state(&self, volume: f64, muted: bool, opacity: f64) {
        let imp = self.imp();
        let mut state = imp.get_render_state();
        state.volume = volume;
        state.muted = muted;
        state.opacity = opacity;
        imp.set_render_state(state);
        self.queue_draw();
    }

    pub fn update_scale(&self, scale: f64) {
        let imp = self.imp();
        let mut state = imp.get_render_state();
        state.scale = scale;
        imp.set_render_state(state);
        self.queue_draw();
    }
}

/// OSD Widget wrapper - manages a window for OSD display
pub struct OsdWidget {
    window: gtk::Window,
    render_widget: OsdRenderWidget,
}

impl OsdWidget {
    /// Create new OSD widget
    pub fn new(scale: f64, composited: bool, application: &gtk::Application) -> Self {
        let window = gtk::Window::new();
        window.set_application(Some(application));
        let render_widget = OsdRenderWidget::new_with_composited(scale, composited);

        // Set widget size
        let width = (OSD_BASE_WIDTH * scale) as i32;
        let height = (OSD_BASE_HEIGHT * scale) as i32;
        render_widget.set_size_request(width, height);

        // Configure window for transparency and OSD behavior
        window.set_decorated(false);
        window.set_resizable(false);
        window.set_default_size(width, height);
        window.set_focus_on_click(false);

        // Apply CSS styling
        window.set_css_classes(&["osd-window"]);
        static CSS_LOADED: std::sync::Once = std::sync::Once::new();
        CSS_LOADED.call_once(|| {
            let css_provider = gtk::CssProvider::new();
            css_provider.load_from_string(
                r"
                .osd-window {
                    background: transparent;
                    border: none;
                    border-width: 0;
                    border-color: transparent;
                    outline: none;
                    outline-width: 0;
                    outline-color: transparent;
                    box-shadow: none;
                }
                ",
            );
            if let Some(display) = gtk::gdk::Display::default() {
                gtk::style_context_add_provider_for_display(
                    &display,
                    &css_provider,
                    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }
        });

        // Add render widget to window
        window.set_child(Some(&render_widget));

        Self {
            window,
            render_widget,
        }
    }

    /// Get the underlying window
    pub fn window(&self) -> &gtk::Window {
        &self.window
    }

    /// Update render state and trigger redraw
    pub fn update_state(&self, volume: f64, muted: bool, opacity: f64) {
        self.render_widget.update_state(volume, muted, opacity);
    }

    /// Update scale and resize the window
    pub fn update_scale(&self, scale: f64) {
        let width = (OSD_BASE_WIDTH * scale) as i32;
        let height = (OSD_BASE_HEIGHT * scale) as i32;
        self.render_widget.set_size_request(width, height);
        self.window.set_default_size(width, height);
        self.render_widget.update_scale(scale);
    }
}
