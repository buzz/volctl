use std::os::raw::c_int;

use glib::object::Cast;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::prelude::{NativeExt, WidgetExt};

use super::{MixerWindow, calculate_mixer_position};
use crate::ui::x11::{
    AtomCollection, X11Context, configure_window_position, send_wm_state_add, set_window_type,
};

// X11
impl MixerWindow {
    /// Position the window at `(x, y)` using quadrant logic.
    pub fn move_x11(&self, x: i32, y: i32) {
        let x11 = self
            .imp()
            .x11_context
            .borrow()
            .as_ref()
            .copied()
            .expect("X11 context");

        if self.is_realized() {
            self.apply_position_x11(x11, x, y);
        } else {
            self.connect_realize(move |window| {
                window.apply_position_x11(x11, x, y);
            });
        }
    }

    /// Apply the calculated position to the X11 window.
    fn apply_position_x11(&self, x11: X11Context, anchor_x: i32, anchor_y: i32) {
        let xid = self.get_xid_x11();
        let (x, y) = calculate_mixer_position(self, anchor_x, anchor_y);

        // Defer the actual X11 move to the next main-loop iteration so the WM has
        // already mapped and placed the window. This prevents the WM from overriding
        // our coordinates with its own smart-placement logic.
        glib::idle_add_once(move || {
            configure_window_position(&x11, xid, x, y);
        });
    }

    pub fn realize_x11(&self) {
        let x11 = self
            .imp()
            .x11_context
            .borrow()
            .as_ref()
            .copied()
            .expect("X11 context");
        let atoms = AtomCollection::new(&x11).expect("Failed to create atoms");
        let xid = self.get_xid_x11();

        self.set_wm_properties_x11(&x11, &atoms, xid);

        let xid_clone = xid;
        let x11_clone = x11;
        let atoms_clone = atoms;

        self.connect_map(move |_| {
            // Use idle_add to defer WM state changes until the window is fully mapped
            glib::idle_add_once(move || {
                send_wm_state_add(
                    x11_clone,
                    xid_clone,
                    atoms_clone._net_wm_state,
                    atoms_clone._net_wm_state_above,
                    atoms_clone._net_wm_state_sticky,
                );
                send_wm_state_add(
                    x11_clone,
                    xid_clone,
                    atoms_clone._net_wm_state,
                    atoms_clone._net_wm_state_skip_taskbar,
                    atoms_clone._net_wm_state_skip_pager,
                );
            });
        });
    }

    fn set_wm_properties_x11(
        &self,
        x11: &X11Context,
        atoms: &AtomCollection,
        xid: gdk_x11::x11::xlib::XID,
    ) {
        // _NET_WM_WINDOW_TYPE = UTILITY
        set_window_type(x11, xid, atoms._net_wm_window_type_utility);

        // _NET_WM_ALLOWED_ACTIONS = CLOSE | ABOVE
        unsafe {
            let actions = [atoms._net_wm_action_close, atoms._net_wm_action_above];
            (x11.xlib().XChangeProperty)(
                x11.display,
                xid,
                atoms._net_wm_allowed_actions,
                gdk_x11::x11::xlib::XA_ATOM,
                32,
                gdk_x11::x11::xlib::PropModeReplace,
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
                gdk_x11::x11::xlib::XA_CARDINAL,
                32,
                gdk_x11::x11::xlib::PropModeReplace,
                &value as *const _ as *const u8,
                1,
            );
        }
    }

    fn get_xid_x11(&self) -> gdk_x11::x11::xlib::XID {
        let surface = self.surface().expect("Failed to get surface.");
        let x11_surface = surface
            .downcast::<gdk_x11::X11Surface>()
            .expect("Failed to get X11 surface.");
        x11_surface.xid()
    }
}
