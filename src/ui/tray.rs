use async_channel::Sender;
use ksni::{Category, MenuItem, Orientation, ToolTip, Tray, menu::StandardItem};
use tracing;

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
    pub use_symbolic_icons: bool,
}

impl Tray for VolctlTray {
    fn icon_name(&self) -> String {
        let state = if self.muted {
            "muted"
        } else {
            let idx = ((self.volume as f32 / MAX_NATURAL_VOL as f32) * 3.0).floor() as usize;
            ["low", "medium", "high"][idx.min(2)]
        };
        if self.use_symbolic_icons {
            format!("audio-volume-{}-symbolic", state)
        } else {
            format!("audio-volume-{}", state)
        }
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
            title: self.get_tooltip_text(),
            description: "".into(),
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
        let use_symbolic = self.use_symbolic_icons;

        vec![
            StandardItem {
                label: "Mute".into(),
                icon_name: if use_symbolic {
                    "audio-volume-muted-symbolic".into()
                } else {
                    "audio-volume-muted".into()
                },
                activate: Box::new(move |_| {
                    if tx_mute.send_blocking(TrayMessage::Mute).is_err() {
                        tracing::warn!(msg = %"Mute", "Channel closed, dropping message");
                    }
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Mixer".into(),
                icon_name: if use_symbolic {
                    "multimedia-volume-control-symbolic".into()
                } else {
                    "multimedia-volume-control".into()
                },
                activate: Box::new(move |_| {
                    if tx_mixer.send_blocking(TrayMessage::ExternalMixer).is_err() {
                        tracing::warn!(msg = %"Mixer", "Channel closed, dropping message");
                    }
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Preferences".into(),
                icon_name: if use_symbolic {
                    "preferences-desktop-symbolic".into()
                } else {
                    "preferences-desktop".into()
                },
                activate: Box::new(move |_| {
                    if tx_prefs.send_blocking(TrayMessage::Preferences).is_err() {
                        tracing::warn!(msg = %"Preferences", "Channel closed, dropping message");
                    }
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "About".into(),
                icon_name: if use_symbolic {
                    "dialog-information-symbolic".into()
                } else {
                    "dialog-information".into()
                },
                activate: Box::new(move |_| {
                    if tx_about.send_blocking(TrayMessage::About).is_err() {
                        tracing::warn!(msg = %"About", "Channel closed, dropping message");
                    }
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit".into(),
                icon_name: if use_symbolic {
                    "application-exit-symbolic".into()
                } else {
                    "application-exit".into()
                },
                activate: Box::new(move |_| {
                    if tx_quit.send_blocking(TrayMessage::Quit).is_err() {
                        tracing::warn!(msg = %"Quit", "Channel closed, dropping message");
                    }
                }),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn activate(&mut self, x: i32, y: i32) {
        if self.tx.send_blocking(TrayMessage::Activate(x, y)).is_err() {
            tracing::warn!(msg = %"Activate", "Channel closed, dropping message");
        }
    }

    fn secondary_activate(&mut self, _x: i32, _y: i32) {
        if self.tx.send_blocking(TrayMessage::ExternalMixer).is_err() {
            tracing::warn!(msg = %"ExternalMixer", "Channel closed, dropping message");
        }
    }

    fn scroll(&mut self, delta: i32, orientation: Orientation) {
        if matches!(orientation, Orientation::Vertical)
            && self.tx.send_blocking(TrayMessage::Scroll(delta)).is_err()
        {
            tracing::warn!(msg = %"Scroll", "Channel closed, dropping message");
        }
    }
}

impl VolctlTray {
    fn get_tooltip_text(&self) -> String {
        let text = format!("Volume: {:.0}%", self.volume_fraction() * 100.0);
        if self.muted {
            format!("{} (muted)", text)
        } else {
            text
        }
    }

    fn volume_fraction(&self) -> f32 {
        self.volume as f32 / MAX_NATURAL_VOL as f32
    }
}
