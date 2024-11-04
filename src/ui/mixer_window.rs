use gtk::{
    prelude::{ButtonExt, GtkWindowExt, WidgetExt},
    Button, Window,
};
use gtk_layer_shell::{Edge, Layer, LayerShell};

use super::{
    wayland::is_wayland_display,
    x11::{x11_get_xid, x11_move_window},
};

pub fn show_mixer(x: i32, y: i32) {
    if is_wayland_display() {
        println!("On wayland");
        show_mixer_wayland();
    } else {
        println!("On X11");
        show_mixer_x11(x, y);
    }
}

fn show_mixer_wayland() {
    let window = Window::new();
    window.set_decorated(false);

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.auto_exclusive_zone_enable();

    window.set_margin(Edge::Right, 32);
    window.set_margin(Edge::Top, 32);

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

    let window_clone = window.clone();
    button.connect_clicked(move |_| {
        println!("Close button clicked");
        window_clone.close();
    });

    window.set_child(Some(&button));
    window.present();
}

fn show_mixer_x11(x: i32, y: i32) {
    let window = Window::new();

    window.set_decorated(false);

    let button = Button::builder()
        .label("Close")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let window_clone = window.clone();
    button.connect_clicked(move |_| {
        println!("Close button clicked");
        window_clone.close();
    });

    window.set_child(Some(&button));
    window.connect_realize(move |window| x11_move_window(x11_get_xid(window), x, y));
    window.present();
}
