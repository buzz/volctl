use gtk::glib;
use ksni::TrayService;

use super::{mixer_window::show_mixer, tray::VolctlTray};

pub enum Message {
    Activate(i32, i32),
    Quit,
}

pub fn create() {
    // https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html#channels
    let (sender, receiver) = async_channel::bounded(1);

    let service = TrayService::new(VolctlTray { sender });
    service.spawn();

    // Listen for messages from the tray thread
    glib::spawn_future_local(async move {
        while let Ok(msg) = receiver.recv().await {
            match msg {
                Message::Activate(x, y) => show_mixer(x, y),
                Message::Quit => std::process::exit(0),
            }
        }
    });
}
