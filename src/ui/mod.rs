mod mixer_window;
mod tray;
mod wayland;
mod x11;

use gtk::glib;
use ksni::TrayService;

use self::{mixer_window::show_mixer, tray::VolctlTray};

pub fn build_ui() {
    // https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html#channels
    let (sender, receiver) = async_channel::bounded(1);

    let service = TrayService::new(VolctlTray { sender });
    service.spawn();

    // Listen for messages from the tray thread
    glib::spawn_future_local(async move {
        while let Ok((x, y)) = receiver.recv().await {
            show_mixer(x, y);
        }
    });
}
