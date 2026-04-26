use std::ffi::CString;
use std::os::raw::c_int;

use gdk_x11::{X11Surface as GdkX11Surface, x11::xlib};
use glib::object::Cast;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::{NativeExt, WidgetExt};

use super::MixerWindow;
use crate::ui::x11::X11Context;

#[derive(Clone, Copy)]
struct AtomCollection {
    _net_wm_state: xlib::Atom,
    _net_wm_state_above: xlib::Atom,
    _net_wm_state_skip_pager: xlib::Atom,
    _net_wm_state_skip_taskbar: xlib::Atom,
    _net_wm_state_sticky: xlib::Atom,
    _net_wm_window_type: xlib::Atom,
    _net_wm_window_type_utility: xlib::Atom,
    _net_wm_bypass_compositor: xlib::Atom,
    _net_wm_allowed_actions: xlib::Atom,
    _net_wm_action_close: xlib::Atom,
    _net_wm_action_above: xlib::Atom,
}

impl AtomCollection {
    fn new(x11: &X11Context) -> Self {
        let intern = |name: &str| {
            let c_name = CString::new(name).expect("atom name");
            unsafe { (x11.xlib().XInternAtom)(x11.display, c_name.as_ptr(), xlib::False) }
        };

        Self {
            _net_wm_state: intern("_NET_WM_STATE"),
            _net_wm_state_above: intern("_NET_WM_STATE_ABOVE"),
            _net_wm_state_skip_pager: intern("_NET_WM_STATE_SKIP_PAGER"),
            _net_wm_state_skip_taskbar: intern("_NET_WM_STATE_SKIP_TASKBAR"),
            _net_wm_state_sticky: intern("_NET_WM_STATE_STICKY"),
            _net_wm_window_type: intern("_NET_WM_WINDOW_TYPE"),
            _net_wm_window_type_utility: intern("_NET_WM_WINDOW_TYPE_UTILITY"),
            _net_wm_bypass_compositor: intern("_NET_WM_BYPASS_COMPOSITOR"),
            _net_wm_allowed_actions: intern("_NET_WM_ALLOWED_ACTIONS"),
            _net_wm_action_close: intern("_NET_WM_ACTION_CLOSE"),
            _net_wm_action_above: intern("_NET_WM_ACTION_ABOVE"),
        }
    }
}

// X11
impl MixerWindow {
    pub fn move_x11(&self, x: i32, y: i32) {
        let xid = self.get_xid_x11();
        let x11 = self.imp().x11_context.borrow().expect("X11 context");

        unsafe {
            let xlib = x11.xlib();
            let mut changes = std::mem::zeroed::<xlib::XWindowChanges>();
            changes.x = x;
            changes.y = y;
            (xlib.XConfigureWindow)(
                x11.display,
                xid,
                (xlib::CWX | xlib::CWY).into(),
                &mut changes,
            );
            (xlib.XFlush)(x11.display);
        }
    }

    pub fn realize_x11(&self) {
        let guard = self.imp().x11_context.borrow();
        let x11 = match guard.as_ref() {
            Some(ctx) => X11Context {
                display: ctx.display,
            },
            None => panic!("X11 context"),
        };
        let atoms = AtomCollection::new(&x11);
        let xid = self.get_xid_x11();

        self.set_wm_properties_x11(&x11, &atoms, xid);

        let xid_clone = xid;
        let x11_clone = x11;
        let atoms_clone = atoms;

        self.connect_map(move |_| {
            // Use idle_add to defer WM state changes until the window is fully mapped
            glib::idle_add_once(move || {
                add_wm_state_x11(
                    x11_clone,
                    &atoms_clone,
                    xid_clone,
                    atoms_clone._net_wm_state_above,
                    atoms_clone._net_wm_state_sticky,
                );
                add_wm_state_x11(
                    x11_clone,
                    &atoms_clone,
                    xid_clone,
                    atoms_clone._net_wm_state_skip_taskbar,
                    atoms_clone._net_wm_state_skip_pager,
                );
            });
        });
    }

    fn set_wm_properties_x11(&self, x11: &X11Context, atoms: &AtomCollection, xid: xlib::XID) {
        // _NET_WM_WINDOW_TYPE = UTILITY
        unsafe {
            let value = atoms._net_wm_window_type_utility;
            (x11.xlib().XChangeProperty)(
                x11.display,
                xid,
                atoms._net_wm_window_type,
                xlib::XA_ATOM,
                32,
                xlib::PropModeReplace,
                &value as *const _ as *const u8,
                1,
            );
        }

        // _NET_WM_ALLOWED_ACTIONS = CLOSE | ABOVE
        unsafe {
            let actions = [atoms._net_wm_action_close, atoms._net_wm_action_above];
            (x11.xlib().XChangeProperty)(
                x11.display,
                xid,
                atoms._net_wm_allowed_actions,
                xlib::XA_ATOM,
                32,
                xlib::PropModeReplace,
                actions.as_ptr() as *const u8,
                actions.len() as c_int,
            );
        }

        // _NET_WM_BYPASS_COMPOSITOR = 2
        unsafe {
            let value: u32 = 2;
            (x11.xlib().XChangeProperty)(
                x11.display,
                xid,
                atoms._net_wm_bypass_compositor,
                xlib::XA_CARDINAL,
                32,
                xlib::PropModeReplace,
                &value as *const _ as *const u8,
                1,
            );
        }
    }

    fn get_xid_x11(&self) -> xlib::XID {
        let surface = self.surface().expect("Failed to get surface.");
        let x11_surface = surface
            .downcast::<GdkX11Surface>()
            .expect("Failed to get X11 surface.");
        x11_surface.xid()
    }
}

fn add_wm_state_x11(
    x11: X11Context,
    atoms: &AtomCollection,
    xid: xlib::XID,
    s1: xlib::Atom,
    s2: xlib::Atom,
) {
    // Build a ClientMessageEvent manually
    // _NET_WM_STATE: op=1(ADD), props=state1,state2, source=1, 0
    const NET_WM_STATE_ADD: i64 = 1;
    const NET_WM_STATE_APP: i64 = 1;

    unsafe {
        let xlib = x11.xlib();
        let mut event: xlib::XEvent = std::mem::zeroed();
        event.client_message.type_ = xlib::ClientMessage;
        event.client_message.window = xid;
        event.client_message.message_type = atoms._net_wm_state;
        event.client_message.format = 32;
        let data = event.client_message.data.as_longs_mut();
        data[0] = NET_WM_STATE_ADD;
        data[1] = s1 as i64;
        data[2] = s2 as i64;
        data[3] = NET_WM_STATE_APP;
        data[4] = 0;

        (xlib.XSendEvent)(
            x11.display,
            xid,
            xlib::False,
            xlib::SubstructureRedirectMask | xlib::StructureNotifyMask,
            &mut event,
        );
        (xlib.XFlush)(x11.display);
    }
}
