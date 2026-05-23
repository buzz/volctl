use std::cell::{Cell, RefCell};
use std::os::raw::{c_int, c_ulong, c_void};
use std::rc::Rc;

use gdk_x11::{X11Surface as GdkX11Surface, x11::xlib};
use gtk::gio::Settings;
use gtk::prelude::*;
use tracing;

use crate::constants::{OSD_BASE_HEIGHT, OSD_BASE_WIDTH, SETTINGS_OSD_MARGIN, SETTINGS_OSD_SCALE};
use crate::ui::osd::controller::OsdStateController;
use crate::ui::osd::widget::OsdWidget;
use crate::ui::utils::Position;
use crate::ui::x11::{
    AtomCollection, X11Context, configure_window_position, set_override_redirect, set_window_type,
    set_wm_states_property,
};

// X11 constants
const SHAPE_BOUNDING: c_int = 0;
const SHAPE_INPUT: c_int = 2;

// Linked via #[link(name = "Xfixes")] at crate root (main.rs)
unsafe extern "C" {
    fn XFixesCreateRegion(
        display: *mut xlib::Display,
        rects: *const c_void,
        nrects: c_int,
    ) -> c_ulong;

    fn XFixesSetWindowShapeRegion(
        display: *mut xlib::Display,
        win: xlib::Window,
        shape: c_int,
        x_off: c_int,
        y_off: c_int,
        region: c_ulong,
    );

    fn XFixesDestroyRegion(display: *mut xlib::Display, region: c_ulong);
}

pub struct X11Surface {
    widget: OsdWidget,
    controller: Rc<OsdStateController>,
    scale: Cell<f64>,
    margin: Cell<i32>,
    atoms: RefCell<Option<AtomCollection>>,
    position: RefCell<Position>,
    composited: bool,
    x11: X11Context,
}

impl X11Surface {
    pub fn new(
        settings: &Settings,
        controller: Rc<OsdStateController>,
        x11_context: X11Context,
        application: &gtk::Application,
    ) -> Self {
        let scale = settings.int(SETTINGS_OSD_SCALE) as f64 / 100.0;
        let width = (OSD_BASE_WIDTH * scale) as i32;
        let height = (OSD_BASE_HEIGHT * scale) as i32;

        let display = gdk::Display::default().unwrap();

        // Detect compositor before creating the widget (affects rendering)
        let composited = display.is_composited();

        let widget = OsdWidget::new(scale, composited, application);

        let window = widget.window();
        window.set_decorated(false);
        window.set_resizable(false);
        window.set_default_size(width, height);
        window.set_focus_on_click(false);

        let atoms = AtomCollection::new(&x11_context);

        let margin = settings.int(SETTINGS_OSD_MARGIN);

        Self {
            widget,
            controller,
            scale: Cell::new(scale),
            margin: Cell::new(margin),
            atoms: RefCell::new(atoms),
            position: RefCell::new(Position::TopLeft),
            composited,
            x11: x11_context,
        }
    }

    /// Get the window XID from the GDK surface.
    fn get_xid(&self) -> Option<xlib::XID> {
        let window = self.widget.window();
        if !window.is_realized() {
            return None;
        }

        let surface = window.surface()?;
        let x11_surface = surface.downcast::<GdkX11Surface>().ok()?;
        Some(x11_surface.xid())
    }

    /// Lazily initialize atoms on first access.
    fn get_atoms(&self) -> Option<AtomCollection> {
        let mut atoms = self.atoms.borrow_mut();
        if atoms.is_none() {
            *atoms = AtomCollection::new(&self.x11);
        }
        *atoms
    }

    fn set_click_through_shape(&self, xid: xlib::XID) {
        unsafe {
            let display = self.x11.display();
            // Clear bounding
            XFixesSetWindowShapeRegion(display, xid, SHAPE_BOUNDING, 0, 0, 0);
            // Empty input region
            let region = XFixesCreateRegion(display, std::ptr::null(), 0);
            XFixesSetWindowShapeRegion(display, xid, SHAPE_INPUT, 0, 0, region);
            XFixesDestroyRegion(display, region);
        }
    }

    fn get_monitor_geometry(&self) -> Option<gdk::Rectangle> {
        let display = gdk::Display::default()?;

        // Try the monitor containing the OSD surface's current position
        if let Some(surface) = self.widget.window().surface()
            && let Some(monitor) = display.monitor_at_surface(&surface)
        {
            return Some(monitor.geometry());
        }

        // Fallback: primary (first) monitor
        let monitors = display.monitors();
        if monitors.n_items() > 0
            && let Some(obj) = monitors.item(0)
            && let Ok(monitor) = obj.downcast::<gdk::Monitor>()
        {
            return Some(monitor.geometry());
        }

        None
    }

    fn calculate_position(&self, position: Position) -> (i32, i32) {
        use crate::ui::utils::{HorizontalPos, VerticalPos};

        let margin = self.margin.get();
        let scale = self.scale.get();
        let width = (OSD_BASE_WIDTH * scale) as i32;
        let height = (OSD_BASE_HEIGHT * scale) as i32;

        let mut xpos = 0;
        let mut ypos = 0;
        let mut swidth = 1;
        let mut sheight = 1;

        if let Some(geometry) = self.get_monitor_geometry() {
            xpos = geometry.x();
            ypos = geometry.y();
            swidth = geometry.width();
            sheight = geometry.height();
        }

        let x = match position.horizontal() {
            HorizontalPos::Left => xpos + margin,
            HorizontalPos::Center => xpos + (swidth - width) / 2,
            HorizontalPos::Right => xpos + swidth - width - margin,
        };

        let y = match position.vertical() {
            VerticalPos::Top => ypos + margin,
            VerticalPos::Center => ypos + (sheight - height) / 2,
            VerticalPos::Bottom => ypos + sheight - height - margin,
        };

        (x, y)
    }
}

impl super::SurfaceBackend for X11Surface {
    fn update_scale(&self, scale: f64) {
        self.scale.set(scale);
        self.widget.update_scale(scale);
    }

    fn update_margin(&self, margin: i32) {
        self.margin.set(margin);
    }

    fn show(&self) {
        let window = self.widget.window();

        if !window.is_realized() {
            gtk::prelude::WidgetExt::realize(window);
        }

        if let Some(xid) = self.get_xid() {
            set_override_redirect(&self.x11, xid);

            if let Some(atoms) = self.get_atoms() {
                set_window_type(
                    &self.x11,
                    xid,
                    &atoms,
                    atoms._net_wm_window_type_notification,
                );
                set_wm_states_property(
                    &self.x11,
                    xid,
                    &atoms,
                    &[
                        atoms._net_wm_state_above,
                        atoms._net_wm_state_skip_taskbar,
                        atoms._net_wm_state_skip_pager,
                        atoms._net_wm_state_sticky,
                    ],
                );
            } else {
                tracing::warn!("Failed to create X11 atoms, skipping WM properties");
            }

            self.set_click_through_shape(xid);

            // Apply position before mapping to avoid flicker
            let position = *self.position.borrow();
            let (x, y) = self.calculate_position(position);
            configure_window_position(&self.x11, xid, x, y);
        }

        let volume = self.controller.get_volume_normalized();
        let muted = self.controller.get_muted();
        let opacity = self.controller.get_opacity();
        self.widget.update_state(volume, muted, opacity);

        // Reset window opacity to fully visible before presenting
        window.set_opacity(1.0);

        window.present();
    }

    fn update_position(&self, position: Position) {
        *self.position.borrow_mut() = position;

        let (x, y) = self.calculate_position(position);

        if let Some(xid) = self.get_xid() {
            configure_window_position(&self.x11, xid, x, y);
        }
    }

    fn begin_fade_out(&self, opacity: f64) {
        let window = self.widget.window();
        window.set_opacity(opacity);

        if opacity <= 0.0 {
            window.set_visible(false);
        }
    }

    fn is_composited(&self) -> bool {
        self.composited
    }

    fn destroy(&self) {
        let window = self.widget.window();
        window.set_visible(false);
        window.destroy();
    }
}
