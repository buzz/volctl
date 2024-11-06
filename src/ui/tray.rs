use async_channel::Sender;
use ksni::{menu::StandardItem, MenuItem, Tray};

use crate::constants::MAX_NATURAL_VOL;

pub enum TrayMessage {
    Activate(i32, i32),
    Quit,
}

#[derive(Debug)]
pub struct VolctlTray {
    pub tx: Sender<TrayMessage>,
    pub volume: u32,
    pub muted: bool,
}

impl Tray for VolctlTray {
    fn icon_name(&self) -> String {
        let state = if self.muted {
            "muted"
        } else {
            let idx = ((self.volume as f32 / MAX_NATURAL_VOL as f32) * 3.0).floor() as usize;
            ["low", "medium", "high"][idx.min(2)]
        };
        format!("audio-volume-{}", state)
    }

    fn title(&self) -> String {
        "volctl".into()
    }

    // On some system trays, `Tray::id` is a required property to avoid unexpected behaviors
    fn id(&self) -> String {
        "volctl".into()
    }

    fn category(&self) -> ksni::Category {
        ksni::Category::Hardware
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let sender = self.tx.clone();

        vec![StandardItem {
            label: "Quit".into(),
            icon_name: "application-exit".into(),
            activate: Box::new(move |_| {
                sender
                    .send_blocking(TrayMessage::Quit)
                    .expect("The channel needs to be open.")
            }),
            ..Default::default()
        }
        .into()]
    }

    fn activate(&mut self, x: i32, y: i32) {
        self.tx
            .send_blocking(TrayMessage::Activate(x, y))
            .expect("The channel needs to be open.");
    }

    fn secondary_activate(&mut self, x: i32, y: i32) {
        println!("ksni: Secondary activate {} {}", x, y);
    }

    fn scroll(&mut self, delta: i32, dir: &str) {
        println!("ksni: Scroll {} {}", delta, dir);
    }
}
