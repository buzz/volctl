use glib::{
    object::Cast,
    subclass::{
        object::ObjectImpl, object::ObjectImplExt, types::ObjectSubclass, types::ObjectSubclassExt,
    },
};

use gtk::{
    prelude::{ButtonExt, GtkWindowExt, WidgetExt},
    subclass::{widget::WidgetImpl, window::WindowImpl},
    Button,
};
use gtk_layer_shell::{Edge, Layer, LayerShell};

use super::{
    wayland::is_wayland_display,
    x11::{x11_get_xid, x11_move_window},
};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MixerWindow {
        // pub(super) settings: OnceCell<Settings>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MixerWindow {
        const NAME: &'static str = "VolctlMixerWindow";
        type Type = super::MixerWindow;
        type ParentType = gtk::Window;
    }

    impl ObjectImpl for MixerWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_visible(false);
        }
    }

    impl WindowImpl for MixerWindow {}

    impl WidgetImpl for MixerWindow {}
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
        if is_wayland_display() {
            self.init_layer_shell();
            self.set_layer(Layer::Overlay);
            self.auto_exclusive_zone_enable();

            self.set_margin(Edge::Right, 32);
            self.set_margin(Edge::Top, 32);

            self.set_anchor(Edge::Left, false);
            self.set_anchor(Edge::Right, true);
            self.set_anchor(Edge::Top, true);
            self.set_anchor(Edge::Bottom, false);
        } else {
            if self.is_realized() {
                self.x11_move(x, y);
            } else {
                self.connect_realize(move |window| {
                    window.x11_move(x, y);
                });
            }
        }
    }

    fn x11_move(&self, x: i32, y: i32) {
        let window = self.upcast_ref::<gtk::Window>();
        let xid = x11_get_xid(window);
        x11_move_window(xid, x, y);
    }
}
