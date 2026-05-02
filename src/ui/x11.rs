use std::ffi::CString;
use std::os::raw::c_int;

use gdk_x11::{
    X11Display,
    x11::xlib::{self, Display, Xlib},
};
use gtk::prelude::*;

use crate::errors::X11Error;

/// Shared X11 display context (thin wrapper around GDK's X11 Display*).
pub struct X11Context {
    pub display: *mut Display,
}

impl X11Context {
    /// Create a new X11 context from the current GDK display.
    pub fn new() -> Result<Self, X11Error> {
        let gdk_display = gdk::Display::default().ok_or(X11Error::NoDisplay)?;
        let x11_display = gdk_display
            .downcast_ref::<X11Display>()
            .ok_or(X11Error::NotX11Display)?;
        let display = unsafe { x11_display.xdisplay() };

        Ok(Self { display })
    }

    /// Get the cached xlib function table (opened once, cached by the x11-dl crate).
    pub fn xlib(&self) -> Xlib {
        Xlib::open().expect("Failed to open Xlib")
    }
}

unsafe impl Send for X11Context {}
unsafe impl Sync for X11Context {}

impl Clone for X11Context {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for X11Context {}

// ---------------------------------------------------------------------------
// AtomCollection
// ---------------------------------------------------------------------------

/// All EWMH atoms used by the OSD and/or mixer window.
#[derive(Clone, Copy)]
pub struct AtomCollection {
    pub _net_wm_state: xlib::Atom,
    pub _net_wm_state_above: xlib::Atom,
    pub _net_wm_state_skip_taskbar: xlib::Atom,
    pub _net_wm_state_skip_pager: xlib::Atom,
    pub _net_wm_state_sticky: xlib::Atom,

    pub _net_wm_window_type: xlib::Atom,
    pub _net_wm_window_type_notification: xlib::Atom,
    pub _net_wm_window_type_utility: xlib::Atom,

    pub _net_wm_bypass_compositor: xlib::Atom,
    pub _net_wm_allowed_actions: xlib::Atom,
    pub _net_wm_action_close: xlib::Atom,
    pub _net_wm_action_above: xlib::Atom,
}

impl AtomCollection {
    /// Intern all atoms on the given display. Returns `None` if any atom lookup fails.
    pub fn new(ctx: &X11Context) -> Option<Self> {
        let intern = |name: &str| -> Option<xlib::Atom> {
            let c_name = CString::new(name).ok()?;
            let atom =
                unsafe { (ctx.xlib().XInternAtom)(ctx.display, c_name.as_ptr(), xlib::False) };
            if atom == 0 { None } else { Some(atom) }
        };

        Some(Self {
            _net_wm_state: intern("_NET_WM_STATE")?,
            _net_wm_state_above: intern("_NET_WM_STATE_ABOVE")?,
            _net_wm_state_skip_taskbar: intern("_NET_WM_STATE_SKIP_TASKBAR")?,
            _net_wm_state_skip_pager: intern("_NET_WM_STATE_SKIP_PAGER")?,
            _net_wm_state_sticky: intern("_NET_WM_STATE_STICKY")?,

            _net_wm_window_type: intern("_NET_WM_WINDOW_TYPE")?,
            _net_wm_window_type_notification: intern("_NET_WM_WINDOW_TYPE_NOTIFICATION")?,
            _net_wm_window_type_utility: intern("_NET_WM_WINDOW_TYPE_UTILITY")?,

            _net_wm_bypass_compositor: intern("_NET_WM_BYPASS_COMPOSITOR")?,
            _net_wm_allowed_actions: intern("_NET_WM_ALLOWED_ACTIONS")?,
            _net_wm_action_close: intern("_NET_WM_ACTION_CLOSE")?,
            _net_wm_action_above: intern("_NET_WM_ACTION_ABOVE")?,
        })
    }
}

// ---------------------------------------------------------------------------
// Shared X11 window helpers
// ---------------------------------------------------------------------------

/// Set `_NET_WM_WINDOW_TYPE` on a window.
pub fn set_window_type(
    ctx: &X11Context,
    xid: xlib::XID,
    atoms: &AtomCollection,
    window_type_atom: xlib::Atom,
) {
    unsafe {
        let c = ctx.xlib();
        (c.XChangeProperty)(
            ctx.display,
            xid,
            atoms._net_wm_window_type,
            xlib::XA_ATOM,
            32,
            xlib::PropModeReplace,
            &window_type_atom as *const _ as *const u8,
            1,
        );
    }
}

/// Set `_NET_WM_STATE` property directly (replaces all states).
pub fn set_wm_states_property(
    ctx: &X11Context,
    xid: xlib::XID,
    atoms: &AtomCollection,
    states: &[xlib::Atom],
) {
    unsafe {
        let c = ctx.xlib();
        (c.XChangeProperty)(
            ctx.display,
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

/// Send `_NET_WM_STATE` ClientMessage to add one or two states.
pub fn send_wm_state_add(
    ctx: X11Context,
    xid: xlib::XID,
    net_wm_state_atom: xlib::Atom,
    s1: xlib::Atom,
    s2: xlib::Atom,
) {
    const NET_WM_STATE_ADD: i64 = 1;
    const NET_WM_STATE_APP: i64 = 1;

    unsafe {
        let c = ctx.xlib();
        let mut event: xlib::XEvent = std::mem::zeroed();
        event.client_message.type_ = xlib::ClientMessage;
        event.client_message.window = xid;
        event.client_message.message_type = net_wm_state_atom;
        event.client_message.format = 32;
        let data = event.client_message.data.as_longs_mut();
        data[0] = NET_WM_STATE_ADD;
        data[1] = s1 as i64;
        data[2] = s2 as i64;
        data[3] = NET_WM_STATE_APP;
        data[4] = 0;

        (c.XSendEvent)(
            ctx.display,
            xid,
            xlib::False,
            xlib::SubstructureRedirectMask | xlib::StructureNotifyMask,
            &mut event,
        );
        (c.XFlush)(ctx.display);
    }
}

/// Set `override_redirect` on a window.
pub fn set_override_redirect(ctx: &X11Context, xid: xlib::XID) {
    unsafe {
        let mut attrs = std::mem::zeroed::<xlib::XSetWindowAttributes>();
        attrs.override_redirect = 1;

        (ctx.xlib().XChangeWindowAttributes)(
            ctx.display,
            xid,
            xlib::CWOverrideRedirect,
            &mut attrs,
        );
    }
}

/// Move a window to `(x, y)` via `XConfigureWindow` + `XFlush`.
pub fn configure_window_position(ctx: &X11Context, xid: xlib::XID, x: i32, y: i32) {
    unsafe {
        let c = ctx.xlib();
        let mut changes = std::mem::zeroed::<xlib::XWindowChanges>();
        changes.x = x;
        changes.y = y;
        (c.XConfigureWindow)(
            ctx.display,
            xid,
            (xlib::CWX | xlib::CWY).into(),
            &mut changes,
        );
        (c.XFlush)(ctx.display);
    }
}
