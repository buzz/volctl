mod app;
mod constants;
mod pulse;
mod ui;

use gdk::prelude::ApplicationExtManual;

use app::Application;

fn main() -> gtk::glib::ExitCode {
    if gtk::init().is_err() {
        panic!("Failed to initialize GTK.");
    }

    let app = Application::default();
    app.run()
}
