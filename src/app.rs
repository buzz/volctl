use gdk::subclass::prelude::{ApplicationImpl, ApplicationImplExt};
use glib::subclass::{
    object::ObjectImpl,
    types::{ObjectSubclass, ObjectSubclassExt},
};
use gtk::{gio, prelude::GtkWindowExt, subclass::prelude::GtkApplicationImpl};

use crate::ui::{mixer_window::MixerWindow, tray_service};

const APP_ID: &str = "org.volctl";

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct Application {
        // pub(super) mixer_window: Rc<MixerWindow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "VolctlApplication";
        type Type = super::Application;
        type ParentType = gtk::Application;
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self) {
            self.parent_activate();
            let app = self.obj();
            tray_service::create(&app);
        }
    }

    impl GtkApplicationImpl for Application {}
}

glib::wrapper! {
  pub struct Application(ObjectSubclass<imp::Application>)
      @extends gio::Application, gtk::Application,
      @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .build()
    }

    pub fn show_mixer(&self, x: i32, y: i32) {
        let mixer_window = MixerWindow::new();
        mixer_window.build_ui(x, y);
        mixer_window.present();
    }

    pub fn quit(&self) {
        // TODO: graceful shutdown
        std::process::exit(0);
    }
}
