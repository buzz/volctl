use gdk::glib::{idle_add, ControlFlow};
use gtk::prelude::{Cast, NativeExt, WidgetExt};
use gtk::Window;

use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt};
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

pub fn x11_move_window(xid: u32, x: i32, y: i32) {
    let (conn, _) = x11rb::connect(None).unwrap();
    let values = ConfigureWindowAux::default().x(x).y(y);

    idle_add(move || {
        match conn.configure_window(xid, &values) {
            Ok(_) => match conn.sync() {
                Ok(_) => {}
                Err(_) => panic!("Sync failed"),
            },
            Err(_) => panic!("Move failed"),
        };
        ControlFlow::Break
    });
}

pub fn x11_get_xid(window: &Window) -> u32 {
    let surface = window.native().unwrap().surface().unwrap();
    let x11_surface = surface.downcast::<gdk_x11::X11Surface>().unwrap();
    x11_surface.xid() as u32
}
