use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gdk_x11::X11Surface as GdkX11Surface;
use gtk::gio::Settings;
use gtk::prelude::*;
use x11rb::connection::Connection;
use x11rb::errors::ReplyError;

use x11rb::protocol::shape::{ConnectionExt as ShapeConnectionExt, SK, SO};
use x11rb::protocol::xproto::ClipOrdering;
use x11rb::protocol::xproto::{
    AtomEnum, ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt, PropMode,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

use crate::constants::{OSD_BASE_HEIGHT, OSD_BASE_WIDTH, OSD_SCREEN_MARGIN, SETTINGS_OSD_SCALE};
use crate::ui::osd::controller::OsdStateController;
use crate::ui::osd::widget::OsdWidget;

pub struct X11Surface {
    widget: OsdWidget,
    controller: Rc<OsdStateController>,
    scale: Cell<f64>,
    conn: Option<Rc<RustConnection>>,
    screen_num: usize,
    xid: Cell<Option<u32>>,
    atoms: Option<AtomCollection>,
    position: RefCell<String>,
    composited: bool,
}

x11rb::atom_manager! {
    pub AtomCollection: AtomCollectionCookie {
        _NET_ACTIVE_WINDOW,
        _NET_WM_WINDOW_TYPE,
        _NET_WM_WINDOW_TYPE_NOTIFICATION,
        _NET_WM_STATE,
        _NET_WM_STATE_ABOVE,
        _NET_WM_STATE_SKIP_TASKBAR,
        _NET_WM_STATE_SKIP_PAGER,
        _NET_WM_STATE_STICKY,
    }
}

impl X11Surface {
    pub fn new(settings: &Settings, controller: Rc<OsdStateController>) -> Self {
        let scale = settings.int(SETTINGS_OSD_SCALE) as f64 / 100.0;
        let width = (OSD_BASE_WIDTH * scale) as i32;
        let height = (OSD_BASE_HEIGHT * scale) as i32;

        // Detect compositor before creating the widget (affects rendering)
        let composited = gdk::Display::default()
            .map(|d| d.is_composited())
            .unwrap_or(false);

        let widget = OsdWidget::new(scale, composited);

        let window = widget.window();
        window.set_decorated(false);
        window.set_resizable(false);
        window.set_default_size(width, height);
        window.set_focus_on_click(false);

        let (conn, screen_num, atoms) = match x11rb::connect(None) {
            Ok((conn, screen_num)) => {
                let conn = Rc::new(conn);
                let atoms = match AtomCollection::new(&conn) {
                    Ok(cookie) => match cookie.reply() {
                        Ok(atoms) => Some(atoms),
                        Err(e) => {
                            eprintln!("Failed to reply AtomCollectionCookie: {}", e);
                            None
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to create AtomCollectionCookie: {}", e);
                        None
                    }
                };
                (Some(conn), screen_num, atoms)
            }
            Err(e) => {
                eprintln!("No X11 connection: {}", e);
                (None, 0, None)
            }
        };

        Self {
            widget,
            controller,
            scale: Cell::new(scale),
            conn,
            screen_num,
            xid: Cell::new(None),
            atoms,
            position: RefCell::new("top-left".to_string()),
            composited,
        }
    }

    fn get_xid(&self) -> Option<u32> {
        if let Some(xid) = self.xid.get() {
            return Some(xid);
        }

        let window = self.widget.window();
        if !window.is_realized() {
            return None;
        }

        let surface = window.surface()?;
        let x11_surface = surface.downcast::<GdkX11Surface>().ok()?;
        let xid = x11_surface.xid() as u32;
        self.xid.set(Some(xid));
        Some(xid)
    }

    fn set_override_redirect(&self, xid: u32) -> Result<(), ReplyError> {
        if let Some(conn) = &self.conn {
            conn.change_window_attributes(
                xid,
                &ChangeWindowAttributesAux::new().override_redirect(1u32),
            )?
            .check()
        } else {
            Ok(())
        }
    }

    fn set_window_type(&self, xid: u32) -> Result<(), ReplyError> {
        if let (Some(conn), Some(atoms)) = (&self.conn, &self.atoms) {
            conn.change_property32(
                PropMode::REPLACE,
                xid,
                atoms._NET_WM_WINDOW_TYPE,
                AtomEnum::ATOM,
                &[atoms._NET_WM_WINDOW_TYPE_NOTIFICATION],
            )?
            .check()
        } else {
            Ok(())
        }
    }

    fn set_click_through_shape(&self, xid: u32) -> Result<(), ReplyError> {
        // Use SHAPE extension to set empty input shape (click-through)
        // SO::SET=0, SK::INPUT=2, empty rectangles = no input area
        if let Some(conn) = &self.conn {
            conn.shape_rectangles(SO::SET, SK::INPUT, ClipOrdering::UNSORTED, xid, 0, 0, &[])?
                .check()
        } else {
            Ok(())
        }
    }

    fn set_wm_states(&self, xid: u32) -> Result<(), ReplyError> {
        // Set _NET_WM_STATE: ABOVE, SKIP_TASKBAR, SKIP_PAGER, STICKY
        // These ensure the window stays above others, doesn't appear in taskbar/pager,
        // and stays on all workspaces.
        if let (Some(conn), Some(atoms)) = (&self.conn, &self.atoms) {
            conn.change_property32(
                PropMode::REPLACE,
                xid,
                atoms._NET_WM_STATE,
                AtomEnum::ATOM,
                &[
                    atoms._NET_WM_STATE_ABOVE,
                    atoms._NET_WM_STATE_SKIP_TASKBAR,
                    atoms._NET_WM_STATE_SKIP_PAGER,
                    atoms._NET_WM_STATE_STICKY,
                ],
            )?
            .check()
        } else {
            Ok(())
        }
    }

    fn get_monitor_geometry(&self) -> Option<gdk::Rectangle> {
        let conn = self.conn.as_ref()?;
        let setup = conn.setup();
        let screen = setup.roots.get(self.screen_num)?;

        let active_atom = conn
            .intern_atom(false, b"_NET_ACTIVE_WINDOW")
            .ok()?
            .reply()
            .ok()?
            .atom;

        let active_reply = conn
            .get_property(false, screen.root, active_atom, AtomEnum::WINDOW, 0, 1)
            .ok()?
            .reply()
            .ok()?;

        if let Some(active_xid) = active_reply.value32()?.next() {
            if let Ok(geom_cookie) = conn.get_geometry(active_xid) {
                if let Ok(geom) = geom_cookie.reply() {
                    let win_center_x = geom.x as i32 + geom.width as i32 / 2;
                    let win_center_y = geom.y as i32 + geom.height as i32 / 2;

                    let display = gdk::Display::default()?;
                    let monitors = display.monitors();
                    for i in 0..monitors.n_items() {
                        if let Some(obj) = monitors.item(i) {
                            if let Ok(monitor) = obj.downcast::<gdk::Monitor>() {
                                let rect = monitor.geometry();
                                if win_center_x >= rect.x()
                                    && win_center_x < rect.x() + rect.width()
                                    && win_center_y >= rect.y()
                                    && win_center_y < rect.y() + rect.height()
                                {
                                    return Some(rect);
                                }
                            }
                        }
                    }
                }
            }
        }

        let display = gdk::Display::default()?;
        // Fallback to first monitor
        let monitors = display.monitors();
        if monitors.n_items() > 0 {
            monitors
                .item(0)
                .and_then(|m| m.downcast::<gdk::Monitor>().ok())
                .map(|m| m.geometry())
        } else {
            None
        }
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
        } else if let Some(conn) = &self.conn {
            // Fallback: query screen dimensions from X11 root window
            if let Some(root) = conn.setup().roots.get(self.screen_num) {
                swidth = root.width_in_pixels as i32;
                sheight = root.height_in_pixels as i32;
            }
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
            if let Err(e) = self.set_override_redirect(xid) {
                eprintln!("Failed to set override_redirect: {}", e);
            }
            if let Err(e) = self.set_window_type(xid) {
                eprintln!("Failed to set window type: {}", e);
            }
            if let Err(e) = self.set_wm_states(xid) {
                eprintln!("Failed to set WM states: {}", e);
            }
            if let Err(e) = self.set_click_through_shape(xid) {
                eprintln!("Failed to set click-through: {}", e);
            }

            // Apply position before mapping to avoid flicker
            let position = self.position.borrow();
            let (x, y) = self.calculate_position(&position);

            let values = ConfigureWindowAux::default().x(x).y(y);
            if let Some(conn) = &self.conn {
                if let Err(err) = conn.configure_window(xid, &values) {
                    eprintln!("Positioning OSD window failed: {}", err);
                } else if let Err(err) = conn.flush() {
                    eprintln!("Flush failed: {}", err);
                }
            }
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
            if let Some(conn) = &self.conn {
                let values = ConfigureWindowAux::default().x(x).y(y);
                // If the connection is available, configure directly instead of
                // deferring to an idle callback.
                match conn.configure_window(xid, &values) {
                    Ok(_) => {
                        if let Err(err) = conn.flush() {
                            eprintln!("Flush failed: {}", err);
                        }
                    }
                    Err(err) => eprintln!("Moving OSD window failed: {}", err),
                };
            }
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
