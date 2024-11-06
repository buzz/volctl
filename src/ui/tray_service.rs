use gtk::glib::clone;
use ksni::TrayService;

use crate::app::Application;

use super::tray::VolctlTray;

pub enum Message {
    Activate(i32, i32),
    Quit,
}

pub fn create(app: &Application) {
    // https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html#channels
    let (sender, receiver) = async_channel::bounded(1);

    let service = TrayService::new(VolctlTray { sender });
    service.spawn();

    // Listen for messages from the tray thread
    glib::spawn_future_local(clone!(
        #[weak]
        app,
        async move {
            while let Ok(msg) = receiver.recv().await {
                match msg {
                    Message::Activate(x, y) => app.show_mixer(x, y),
                    Message::Quit => app.quit(),
                }
            }
        }
    ));
}
