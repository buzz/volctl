use async_channel::Sender;
use ksni::{menu::StandardItem, MenuItem, Tray};

#[derive(Debug)]
pub struct VolctlTray {
    pub sender: Sender<(i32, i32)>,
}

impl Tray for VolctlTray {
    fn icon_name(&self) -> String {
        "help-about".into()
    }

    fn title(&self) -> String {
        "Title".into()
    }

    // On some system trays, `Tray::id` is a required property to avoid unexpected behaviors
    fn id(&self) -> String {
        "volctl".into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![StandardItem {
            label: "Exit".into(),
            icon_name: "application-exit".into(),
            activate: Box::new(|_| std::process::exit(0)),
            ..Default::default()
        }
        .into()]
    }

    fn activate(&mut self, x: i32, y: i32) {
        self.sender
            .send_blocking((x, y))
            .expect("The channel needs to be open.");
    }

    fn secondary_activate(&mut self, x: i32, y: i32) {
        println!("ksni: Secondary activate {} {}", x, y);
    }

    fn scroll(&mut self, delta: i32, dir: &str) {
        println!("ksni: Scroll {} {}", delta, dir);
    }
}
