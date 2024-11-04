mod ui;

use gtk::{glib, prelude::*, Application};

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id("com.volctl").build();

    app.connect_activate(|_| ui::build_ui());

    // Prevent GTK main loop from exiting without window.
    let _guard = app.hold();

    app.run()
}
