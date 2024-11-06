use gtk::gio;

const APP_ID: &str = "org.volctl";

mod imp {
    use gdk::subclass::prelude::ApplicationImpl;
    use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
    use gtk::subclass::prelude::GtkApplicationImpl;

    use crate::ui::tray_service;

    #[derive(Debug, Default)]
    pub struct Application {
        // pub(super) settings: OnceCell<Settings>,
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
            tray_service::create();
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
}
