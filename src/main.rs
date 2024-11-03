use gtk::{glib, prelude::*, Application, ApplicationWindow, Button};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use ksni::{menu::StandardItem, MenuItem, Tray};

#[derive(Debug)]
struct VolctlTray {
    sender: async_channel::Sender<(i32, i32)>,
}

impl Tray for VolctlTray {
    fn icon_name(&self) -> String {
        "help-about".into()
    }

    fn title(&self) -> String {
        "Title".into()
    }

    // **NOTE**: On some system trays, [`Tray::id`] is a required property to avoid unexpected behaviors
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
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
        println!("ksni: Activate {} {}", x, y);
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

fn main() -> glib::ExitCode {
    match get_session_type() {
        SessionType::WAYLAND => println!("Wayland session detected"),
        SessionType::X11 => println!("X11 session detected"),
    }

    let application = Application::builder()
        .application_id("com.github.gtk-rs.examples.basic")
        .build();
    application.connect_activate(build_ui);

    // Prevent GTK main loop from exiting without window.
    let _hold_guard = application.hold();

    application.run()
}

fn build_ui(app: &Application) {
    // https://gtk-rs.org/gtk4-rs/stable/latest/book/main_event_loop.html#channels
    // let (tx, rx) = mpsc::channel();
    let (sender, receiver) = async_channel::bounded(1);

    let service = ksni::TrayService::new(VolctlTray { sender });
    // let handle = service.handle();
    service.spawn();

    // Listen for messages from the tray thread
    glib::spawn_future_local(glib::clone!(
        #[weak]
        app,
        async move {
            println!("gtk: awaiting...");
            while let Ok((x, y)) = receiver.recv().await {
                println!("gtk: Activate {} {}", x, y);
                show_popup(&app, x, y);
            }
        }
    ));
}

fn show_popup(app: &Application, x: i32, y: i32) {
    // move window manually X11
    // wayland??
    // https://discourse.gnome.org/t/how-to-center-gtkwindows-in-gtk4/3112/13

    let window = ApplicationWindow::new(app);
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.auto_exclusive_zone_enable();
    window.set_decorated(false);

    window.set_margin(Edge::Left, 40);
    window.set_margin(Edge::Right, 40);
    window.set_margin(Edge::Top, 32);
    window.set_margin(Edge::Bottom, 20);

    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, false);

    let button = Button::builder()
        .label("Close")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    window.set_child(Some(&button));

    button.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| {
            println!("Close button clicked");
            window.destroy();
        }
    ));

    window.present();
}

enum SessionType {
    X11,
    WAYLAND,
}

fn get_session_type() -> SessionType {
    match std::env::var("XDG_SESSION_TYPE") {
        Ok(xdg_session_type) => match xdg_session_type.as_str() {
            "wayland" => SessionType::WAYLAND,
            "x11" => SessionType::X11,
            _ => panic!("Unknown XDG_SESSION_TYPE={}", xdg_session_type),
        },
        Err(_) => panic!("XDG_SESSION_TYPE not set!"),
    }
}
