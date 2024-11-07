use std::rc::Rc;

use gdk_x11::X11Surface;
use glib::object::Cast;
use glib::subclass::object::{ObjectImpl, ObjectImplExt};
use glib::subclass::types::{ObjectSubclass, ObjectSubclassExt};
use glib::{idle_add, ControlFlow};
use gtk::prelude::{ButtonExt, GtkWindowExt, NativeExt, WidgetExt};
use gtk::subclass::widget::WidgetImplExt;
use gtk::subclass::{widget::WidgetImpl, window::WindowImpl};
use gtk::Button;
use gtk_layer_shell::{Edge, Layer, LayerShell};
use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::protocol::xproto::{
    AtomEnum, ClientMessageEvent, ConfigureWindowAux, ConnectionExt, EventMask, PropMode,
    CLIENT_MESSAGE_EVENT,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::x11_utils::Serialize;

use crate::ui::utils::is_wayland;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MixerWindow {}

    #[glib::object_subclass]
    impl ObjectSubclass for MixerWindow {
        const NAME: &'static str = "VolctlMixerWindow";
        type Type = super::MixerWindow;
        type ParentType = gtk::Window;
    }

    impl ObjectImpl for MixerWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.set_visible(false);
        }
    }

    impl WindowImpl for MixerWindow {}

    impl WidgetImpl for MixerWindow {
        fn realize(&self) {
            self.parent_realize();

            if !is_wayland() {
                self.obj().realize_x11();
            }
        }
    }
}

glib::wrapper! {
  pub struct MixerWindow(ObjectSubclass<imp::MixerWindow>)
      @extends gtk::Window, gtk::Widget,
      @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl MixerWindow {
    pub fn new() -> Self {
        let window: MixerWindow = glib::Object::builder()
            .property("decorated", false)
            .property("resizable", false)
            .property("deletable", false)
            .build();

        // TODO: set some hints: https://github.com/Elvyria/Mixxc/blob/master/src/proto/x.rs

        window
    }

    pub fn build_ui(&self) {
        let button = Button::builder()
            .label("Close")
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let window_clone = self.clone();
        button.connect_clicked(move |_| {
            println!("Close button clicked");
            window_clone.close();
        });

        self.set_child(Some(&button));
    }

    pub fn move_(&self, x: i32, y: i32) {
        if is_wayland() {
            self.move_wayland(x, y);
        } else {
            if self.is_realized() {
                self.move_x11(x, y);
            } else {
                self.connect_realize(move |window| {
                    window.move_x11(x, y);
                });
            }
        }
    }
}

// X11
impl MixerWindow {
    fn move_x11(&self, x: i32, y: i32) {
        let xid = self.get_xid_x11(&self.get_surface_x11());
        let conn = self.get_connection_x11();
        let values = ConfigureWindowAux::default().x(x).y(y);

        idle_add(move || {
            match conn.configure_window(xid, &values) {
                Ok(_) => {
                    if let Err(err) = conn.flush() {
                        eprintln!("Flush failed: {}", err);
                    }
                }
                Err(err) => eprintln!("Moving window failed: {}", err),
            };
            ControlFlow::Break
        });
    }

    fn realize_x11(&self) {
        let surface = self.get_surface_x11();
        let conn = self.get_connection_x11();
        let atoms = AtomCollection::new(&conn)
            .expect("Failed to create AtomCollectionCookie.")
            .reply()
            .expect("Failed to create AtomCollectionCookie.");
        let xid = self.get_xid_x11(&surface);

        self.set_wm_properties_x11(&conn, atoms, xid)
            .expect("Failed to set WM properties.");

        let conn = Rc::new(conn);

        self.connect_map({
            let conn = conn.clone();
            let win_clone = self.clone();

            move |_| {
                win_clone
                    .add_wm_state_x11(
                        conn.as_ref(),
                        xid,
                        atoms,
                        atoms._NET_WM_STATE_ABOVE,
                        atoms._NET_WM_STATE_STICKY,
                    )
                    .expect("Failed to add WM state.");
                win_clone
                    .add_wm_state_x11(
                        conn.as_ref(),
                        xid,
                        atoms,
                        atoms._NET_WM_STATE_SKIP_TASKBAR,
                        atoms._NET_WM_STATE_SKIP_PAGER,
                    )
                    .expect("Failed to add WM state.");
            }
        });
    }

    fn set_wm_properties_x11(
        &self,
        conn: &impl Connection,
        atoms: AtomCollection,
        xid: u32,
    ) -> Result<(), ReplyError> {
        conn.change_property32(
            PropMode::REPLACE,
            xid,
            atoms._NET_WM_WINDOW_TYPE,
            AtomEnum::ATOM,
            &[atoms._NET_WM_WINDOW_TYPE_UTILITY],
        )?
        .check()?;

        conn.change_property32(
            PropMode::REPLACE,
            xid,
            atoms._NET_WM_ALLOWED_ACTIONS,
            AtomEnum::ATOM,
            &[atoms._NET_WM_ACTION_CLOSE, atoms._NET_WM_ACTION_ABOVE],
        )?
        .check()?;

        conn.change_property32(
            PropMode::REPLACE,
            xid,
            atoms._NET_WM_BYPASS_COMPOSITOR,
            AtomEnum::CARDINAL,
            &[2],
        )?
        .check()?;

        Ok(())
    }

    fn add_wm_state_x11(
        &self,
        conn: &impl Connection,
        xid: u32,
        atoms: AtomCollection,
        s1: u32,
        s2: u32,
    ) -> Result<(), ReplyError> {
        const _NET_WM_STATE_ADD: u32 = 1;
        const _NET_WM_STATE_APP: u32 = 1;

        let event = ClientMessageEvent {
            response_type: CLIENT_MESSAGE_EVENT,
            format: 32,
            sequence: 0,
            window: xid,
            type_: atoms._NET_WM_STATE,
            data: [_NET_WM_STATE_ADD, s1, s2, _NET_WM_STATE_APP, 0].into(),
        };

        conn.send_event(
            false,
            xid,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::STRUCTURE_NOTIFY,
            event.serialize(),
        )?
        .check()
    }

    fn get_xid_x11(&self, surface: &X11Surface) -> u32 {
        surface.xid() as u32
    }

    fn get_surface_x11(&self) -> X11Surface {
        self.surface()
            .expect("Failed to get surface.")
            .downcast::<X11Surface>()
            .expect("Failed to get X11 surface.")
    }

    fn get_connection_x11(&self) -> RustConnection {
        x11rb::connect(None).expect("No X11 connection.").0
    }
}

// Wayland
impl MixerWindow {
    fn move_wayland(&self, x: i32, y: i32) {
        self.init_layer_shell();
        self.set_layer(Layer::Overlay);
        self.auto_exclusive_zone_enable();

        self.set_margin(Edge::Right, 32);
        self.set_margin(Edge::Top, 32);

        self.set_anchor(Edge::Left, false);
        self.set_anchor(Edge::Right, true);
        self.set_anchor(Edge::Top, true);
        self.set_anchor(Edge::Bottom, false);
    }
}

x11rb::atom_manager! {
    pub AtomCollection: AtomCollectionCookie {
        _NET_WM_STATE,
        _NET_WM_STATE_ABOVE,
        _NET_WM_STATE_SKIP_PAGER,
        _NET_WM_STATE_SKIP_TASKBAR,
        _NET_WM_STATE_STICKY,

        _NET_WM_WINDOW_TYPE,
        _NET_WM_WINDOW_TYPE_UTILITY,

        _NET_WM_BYPASS_COMPOSITOR,

        _NET_WM_ALLOWED_ACTIONS,
        _NET_WM_ACTION_CLOSE,
        _NET_WM_ACTION_ABOVE,
    }
}
