use std::cell::{OnceCell, RefCell};

use gdk::prelude::ApplicationExtManual;
use gdk::subclass::prelude::{ApplicationImpl, ApplicationImplExt};
use glib::clone;
use glib::subclass::object::{ObjectImpl, ObjectImplExt};
use glib::subclass::types::{ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt};
use gtk::gio;
use gtk::prelude::{GtkWindowExt, SettingsExt, WidgetExt};
use gtk::subclass::prelude::GtkApplicationImpl;
use ksni::TrayService;

use super::pulse::Pulse;
use super::ui::{
    mixer_window::MixerWindow,
    tray::{TrayMessage, VolctlTray},
};

const APP_ID: &str = "org.volctl";

mod imp {
    use std::time::Duration;

    use super::*;

    pub struct Application {
        pub(super) hold_guard: RefCell<Option<gio::ApplicationHoldGuard>>,
        pub(super) settings: OnceCell<gio::Settings>,
        pub(super) mixer_window: OnceCell<MixerWindow>,
        pub(super) first_volume_update: RefCell<bool>,
        pub(super) pulse: RefCell<Pulse>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "VolctlApplication";
        type Type = super::Application;
        type ParentType = gtk::Application;
    }

    impl ObjectImpl for Application {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl ApplicationImpl for Application {
        fn activate(&self) {
            self.parent_activate();
            let app = self.obj();

            // Prevent GTK main loop from exiting without window.
            *self.hold_guard.borrow_mut() = Some(app.hold());

            self.settings.get().unwrap().connect_changed(
                Some("changed::mouse-wheel-step"),
                |_, _| {
                    println!("Settings changed");
                },
            );

            // Connect to PulseAudio
            let mut pulse = self.pulse.borrow_mut();
            pulse.connect();

            // Periodically update widgets from PulseAudio
            glib::timeout_add_local(
                Duration::from_millis(1000 / 30),
                clone!(
                    #[strong]
                    app,
                    move || {
                        app.update();
                        glib::ControlFlow::Continue
                    }
                ),
            );

            // https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html#channels
            let (sender, receiver) = async_channel::bounded(1);

            // Start tray service
            let tray_service = TrayService::new(VolctlTray { sender });
            tray_service.spawn();

            // Listen for messages from the tray thread
            glib::spawn_future_local(clone!(
                #[weak]
                app,
                async move {
                    while let Ok(msg) = receiver.recv().await {
                        match msg {
                            TrayMessage::Activate(x, y) => app.toggle_mixer(x, y),
                            TrayMessage::Quit => app.request_quit(),
                        }
                    }
                }
            ));

            let mixer_window = self.mixer_window.get().unwrap();
            mixer_window.build_ui();
        }
    }

    impl GtkApplicationImpl for Application {}

    impl Default for Application {
        fn default() -> Self {
            Self {
                hold_guard: RefCell::from(None),
                settings: OnceCell::from(gio::Settings::with_path("apps.volctl", "/apps/volctl/")),
                mixer_window: OnceCell::from(MixerWindow::new()),
                first_volume_update: RefCell::from(false),
                pulse: RefCell::from(Pulse::new()),
            }
        }
    }
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

    pub fn toggle_mixer(&self, x: i32, y: i32) {
        let window = self.imp().mixer_window.get().unwrap();
        if window.get_visible() {
            println!("Hide");
            window.set_visible(false);
        } else {
            println!("Show {} {}", x, y);
            window.move_(x, y);
            window.present();
        }
    }

    fn request_quit(&self) {
        let imp = self.imp();

        // Close pulse.
        imp.pulse.borrow_mut().cleanup();

        // Close mixer window.
        if let Some(win) = imp.mixer_window.get() {
            win.destroy();
        }

        // Discard application hold guard.
        *imp.hold_guard.borrow_mut() = None;
    }

    fn update(&self) {
        let mut pulse = self.imp().pulse.borrow_mut();

        if pulse.update() {}
    }
}
