// Link libXfixes for click-through OSD windows (XFixesSetWindowShapeRegion etc.)
#[cfg(target_os = "linux")]
#[link(name = "Xfixes")]
unsafe extern "C" {}

mod app;
mod constants;
mod errors;
mod pulse;
mod ui;

use gdk::prelude::ApplicationExtManual;

use app::Application;

fn main() -> gtk::glib::ExitCode {
    tracing_subscriber::fmt().init();

    if gtk::init().is_err() {
        panic!("Failed to initialize GTK.");
    }

    let app = Application::default();
    app.run()
}
