// Link libXfixes for click-through OSD windows (XFixesSetWindowShapeRegion etc.)
#[cfg(all(target_os = "linux", feature = "x11"))]
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

    gtk::init().expect("Failed to initialize GTK.");

    let app = Application::default();
    app.run()
}
