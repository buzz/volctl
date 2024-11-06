mod app;
mod ui;

use gdk::prelude::ApplicationExtManual;

use app::Application;

fn main() -> gtk::glib::ExitCode {
    let app = Application::new();

    // Prevent GTK main loop from exiting without window.
    let _guard = app.hold();

    app.run()
}
