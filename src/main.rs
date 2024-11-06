mod app;
mod pulse;
mod ui;

use gdk::prelude::ApplicationExtManual;

use app::Application;

fn main() -> gtk::glib::ExitCode {
    if let Err(_) = gtk::init() {
        panic!("Failed to initialize GTK.");
    }

    let app = Application::new();

    // Prevent GTK main loop from exiting without window.
    // let _guard = app.hold();

    app.run()
}
