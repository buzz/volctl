use async_channel::Sender;
use ksni::{menu::StandardItem, MenuItem, Tray};

use super::tray_service::Message;

#[derive(Debug)]
pub struct VolctlTray {
    pub sender: Sender<Message>,
}

impl Tray for VolctlTray {
    fn icon_name(&self) -> String {
        "help-about".into()
    }

    fn title(&self) -> String {
        "volctl".into()
    }

    // On some system trays, `Tray::id` is a required property to avoid unexpected behaviors
    fn id(&self) -> String {
        "volctl".into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let sender = self.sender.clone();

        vec![StandardItem {
            label: "Quit".into(),
            icon_name: "application-exit".into(),
            activate: Box::new(move |_| {
                sender
                    .send_blocking(Message::Quit)
                    .expect("The channel needs to be open.")
            }),
            ..Default::default()
        }
        .into()]
    }

    fn activate(&mut self, x: i32, y: i32) {
        self.sender
            .send_blocking(Message::Activate(x, y))
            .expect("The channel needs to be open.");
    }

    fn secondary_activate(&mut self, x: i32, y: i32) {
        println!("ksni: Secondary activate {} {}", x, y);
    }

    fn scroll(&mut self, delta: i32, dir: &str) {
        println!("ksni: Scroll {} {}", delta, dir);
    }
}
