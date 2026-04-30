use std::error::Error;

use gtk::{AlertDialog, Window, glib};

/// Show an error dialog. Safe to call from any thread.
///
/// Uses `MainContext::invoke` to marshal to the GTK main thread,
/// so callers from the Pulse thread, tray thread, or async tasks
/// don't need to worry about threading.
pub fn show_error<E: Error + 'static>(err: &E) {
    let message = err.to_string();
    let title = std::any::type_name::<E>()
        .rsplit("::")
        .next()
        .unwrap_or("Error");

    glib::MainContext::default().invoke(move || {
        let dialog = AlertDialog::default();
        dialog.set_message(&message);
        dialog.set_detail(&format!("{} - volctl", title));
        dialog.set_buttons(&["OK"]);
        dialog.show(None::<&Window>);
    });
}
