use async_channel::Sender;
use ksni::{menu::StandardItem, Category, MenuItem, ToolTip, Tray};

use crate::constants::MAX_NATURAL_VOL;

pub enum TrayMessage {
    About,
    Activate(i32, i32),
    ExternalMixer,
    Mute,
    Preferences,
    Quit,
    Scroll(i32),
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

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            icon_name: "".into(),
            icon_pixmap: [].to_vec(),
            title: "Volume".into(),
            description: self.get_tooltip_markup(),
        }
    }

    fn category(&self) -> Category {
        Category::Hardware
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let tx_mute = self.tx.clone();
        let tx_mixer = self.tx.clone();
        let tx_prefs = self.tx.clone();
        let tx_about = self.tx.clone();
        let tx_quit = self.tx.clone();

        vec![
            StandardItem {
                label: "Mute".into(),
                icon_name: "audio-volume-muted".into(),
                activate: Box::new(move |_| {
                    if tx_mute.send_blocking(TrayMessage::Mute).is_err() {
                        eprintln!("Failed to send Mute message, channel closed");
                    }
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Mixer".into(),
                icon_name: "multimedia-volume-control".into(),
                activate: Box::new(move |_| {
                    if tx_mixer.send_blocking(TrayMessage::ExternalMixer).is_err() {
                        eprintln!("Failed to send Mixer message, channel closed");
                    }
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Preferences".into(),
                icon_name: "preferences-desktop".into(),
                activate: Box::new(move |_| {
                    if tx_prefs.send_blocking(TrayMessage::Preferences).is_err() {
                        eprintln!("Failed to send Preferences message, channel closed");
                    }
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "About".into(),
                icon_name: "dialog-information".into(),
                activate: Box::new(move |_| {
                    if tx_about.send_blocking(TrayMessage::About).is_err() {
                        eprintln!("Failed to send About message, channel closed");
                    }
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(move |_| {
                    if tx_quit.send_blocking(TrayMessage::Quit).is_err() {
                        eprintln!("Failed to send Quit message, channel closed");
                    }
                }),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn activate(&mut self, x: i32, y: i32) {
        if self.tx.send_blocking(TrayMessage::Activate(x, y)).is_err() {
            eprintln!("Failed to send Activate message, channel closed");
        }
    }

    fn secondary_activate(&mut self, x: i32, y: i32) {
        println!("ksni: Secondary activate {} {}", x, y);
    }

    fn scroll(&mut self, delta: i32, dir: &str) {
        if dir == "vertical" && self.tx.send_blocking(TrayMessage::Scroll(delta)).is_err() {
            eprintln!("Failed to send Scroll message, channel closed");
        }
    }
}

impl VolctlTray {
    fn get_tooltip_markup(&self) -> String {
        let text = format!("{:.0}%", self.volume_fraction() * 100.0);
        if self.muted {
            format!("{} <span weight=\"bold\">(muted)</span>", text)
        } else {
            text
        }
    }

    fn volume_fraction(&self) -> f32 {
        self.volume as f32 / MAX_NATURAL_VOL as f32
    }
}
