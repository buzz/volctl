use std::cell::{Cell, RefCell};
use std::ffi::CString;
use std::os::raw::{c_int, c_ulong, c_void};
use std::rc::Rc;

use gdk_x11::{X11Surface as GdkX11Surface, x11::xlib};
use gtk::gio::Settings;
use gtk::prelude::*;

use crate::constants::{OSD_BASE_HEIGHT, OSD_BASE_WIDTH, OSD_SCREEN_MARGIN, SETTINGS_OSD_SCALE};
use crate::ui::osd::controller::OsdStateController;
use crate::ui::osd::widget::OsdWidget;
use crate::ui::x11::X11Context;

// X11 constants
const SHAPE_BOUNDING: c_int = 0;
const SHAPE_INPUT: c_int = 2;

#[link(name = "Xfixes")]
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

#[derive(Clone, Copy)]
struct AtomCollection {
    _net_wm_window_type: xlib::Atom,
    _net_wm_window_type_notification: xlib::Atom,
    _net_wm_state: xlib::Atom,
    _net_wm_state_above: xlib::Atom,
    _net_wm_state_skip_taskbar: xlib::Atom,
    _net_wm_state_skip_pager: xlib::Atom,
    _net_wm_state_sticky: xlib::Atom,
}

impl AtomCollection {
    fn new(x11_context: &X11Context) -> Option<Self> {
        let intern = |name: &str| {
            let c_name = CString::new(name).ok()?;
            let atom = unsafe {
                (x11_context.xlib().XInternAtom)(x11_context.display, c_name.as_ptr(), xlib::False)
            };
            if atom == 0 { None } else { Some(atom) }
        };

        Some(Self {
            _net_wm_window_type: intern("_NET_WM_WINDOW_TYPE")?,
            _net_wm_window_type_notification: intern("_NET_WM_WINDOW_TYPE_NOTIFICATION")?,
            _net_wm_state: intern("_NET_WM_STATE")?,
            _net_wm_state_above: intern("_NET_WM_STATE_ABOVE")?,
            _net_wm_state_skip_taskbar: intern("_NET_WM_STATE_SKIP_TASKBAR")?,
            _net_wm_state_skip_pager: intern("_NET_WM_STATE_SKIP_PAGER")?,
            _net_wm_state_sticky: intern("_NET_WM_STATE_STICKY")?,
        })
    }
}

pub struct X11Surface {
    widget: OsdWidget,
    controller: Rc<OsdStateController>,
    scale: Cell<f64>,
    atoms: RefCell<Option<AtomCollection>>,
    position: RefCell<String>,
    composited: bool,
    x11: X11Context,
}

impl X11Surface {
    pub fn new(
        settings: &Settings,
        controller: Rc<OsdStateController>,
        x11_context: X11Context,
    ) -> Self {
        let scale = settings.int(SETTINGS_OSD_SCALE) as f64 / 100.0;
        let width = (OSD_BASE_WIDTH * scale) as i32;
        let height = (OSD_BASE_HEIGHT * scale) as i32;

        let display = gdk::Display::default().unwrap();

        // Detect compositor before creating the widget (affects rendering)
        let composited = display.is_composited();

        let widget = OsdWidget::new(scale, composited);

        let window = widget.window();
        window.set_decorated(false);
        window.set_resizable(false);
        window.set_default_size(width, height);
        window.set_focus_on_click(false);

        let atoms = AtomCollection::new(&x11_context);

        Self {
            widget,
            controller,
            scale: Cell::new(scale),
            atoms: RefCell::new(atoms),
            position: RefCell::new("top-left".to_string()),
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
        let xid = x11_surface.xid();
        Some(xid)
    }

    /// Lazily initialize atoms on first access.
    fn get_atoms(&self) -> AtomCollection {
        let mut atoms = self.atoms.borrow_mut();
        if atoms.is_none() {
            *atoms = AtomCollection::new(&self.x11);
        }
        *atoms.as_ref().expect("atoms initialized above")
    }

    fn set_override_redirect(&self, xid: xlib::XID) {
        unsafe {
            let mut attrs = std::mem::zeroed::<xlib::XSetWindowAttributes>();
            attrs.override_redirect = 1;

            (self.x11.xlib().XChangeWindowAttributes)(
                self.x11.display,
                xid,
                xlib::CWOverrideRedirect,
                &mut attrs,
            );
        }
    }

    fn set_window_type(&self, xid: xlib::XID) {
        let atoms = self.get_atoms();
        let value = atoms._net_wm_window_type_notification;
        unsafe {
            (self.x11.xlib().XChangeProperty)(
                self.x11.display,
                xid,
                atoms._net_wm_window_type,
                xlib::XA_ATOM,
                32, // 32-bit atoms
                xlib::PropModeReplace,
                &value as *const _ as *const u8,
                1,
            );
        }
    }

    fn set_click_through_shape(&self, xid: xlib::XID) {
        unsafe {
            // Clear bounding
            XFixesSetWindowShapeRegion(self.x11.display, xid, SHAPE_BOUNDING, 0, 0, 0);
            // Empty input region
            let region = XFixesCreateRegion(self.x11.display, std::ptr::null(), 0);
            XFixesSetWindowShapeRegion(self.x11.display, xid, SHAPE_INPUT, 0, 0, region);
            XFixesDestroyRegion(self.x11.display, region);
        }
    }

    fn set_wm_states(&self, xid: xlib::XID) {
        let atoms = self.get_atoms();
        let states = [
            atoms._net_wm_state_above,
            atoms._net_wm_state_skip_taskbar,
            atoms._net_wm_state_skip_pager,
            atoms._net_wm_state_sticky,
        ];
        unsafe {
            (self.x11.xlib().XChangeProperty)(
                self.x11.display,
                xid,
                atoms._net_wm_state,
                xlib::XA_ATOM,
                32,
                xlib::PropModeReplace,
                states.as_ptr() as *const u8,
                states.len() as c_int,
            );
        }
    }

    fn configure_position(&self, xid: xlib::XID, x: i32, y: i32) {
        unsafe {
            let mut changes = std::mem::zeroed::<xlib::XWindowChanges>();
            changes.x = x;
            changes.y = y;
            (self.x11.xlib().XConfigureWindow)(
                self.x11.display,
                xid,
                (xlib::CWX | xlib::CWY).into(),
                &mut changes,
            );
            (self.x11.xlib().XFlush)(self.x11.display);
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

    fn calculate_position(&self, position: &str) -> (i32, i32) {
        let margin = OSD_SCREEN_MARGIN;
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

        let parts: Vec<&str> = position.split('-').collect();
        if parts.len() != 2 {
            return (xpos + margin, ypos + margin);
        }

        let yname = parts[0];
        let xname = parts[1];

        let x = match xname {
            "left" => xpos + margin,
            "center" => xpos + (swidth - width) / 2,
            "right" => xpos + swidth - width - margin,
            _ => xpos + margin,
        };

        let y = match yname {
            "top" => ypos + margin,
            "center" => ypos + (sheight - height) / 2,
            "bottom" => ypos + sheight - height - margin,
            _ => ypos + margin,
        };

        (x, y)
    }
}

impl super::SurfaceBackend for X11Surface {
    fn update_scale(&self, scale: f64) {
        self.scale.set(scale);
        self.widget.update_scale(scale);
    }

    fn show(&self) {
        let window = self.widget.window();

        if !window.is_realized() {
            gtk::prelude::WidgetExt::realize(window);
        }

        if let Some(xid) = self.get_xid() {
            self.set_override_redirect(xid);
            self.set_window_type(xid);
            self.set_wm_states(xid);
            self.set_click_through_shape(xid);

            // Apply position before mapping to avoid flicker
            let position = self.position.borrow();
            let (x, y) = self.calculate_position(&position);
            self.configure_position(xid, x, y);
        }

        let volume = self.controller.get_volume_normalized();
        let muted = self.controller.get_muted();
        let opacity = self.controller.get_opacity();
        self.widget.update_state(volume, muted, opacity);

        // Reset window opacity to fully visible before presenting
        window.set_opacity(1.0);

        window.present();
    }

    fn update_position(&self, position: &str) {
        *self.position.borrow_mut() = position.to_string();

        let (x, y) = self.calculate_position(position);

        if let Some(xid) = self.get_xid() {
            self.configure_position(xid, x, y);
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
