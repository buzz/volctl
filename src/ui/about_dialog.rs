use gtk::{AboutDialog, License, prelude::*};

/// Application metadata constants
const PROGRAM_NAME: &str = "Volume Control";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const COPYRIGHT: &str = "(c) buzz";
const LICENSE_TYPE: License = License::Gpl30;
const COMMENTS: &str = "Per-application volume control for GNU/Linux desktops";
const WEBSITE: &str = "https://buzz.github.io/volctl/";
const LOGO_ICON_NAME: &str = "audio-volume-high";

pub fn new() -> AboutDialog {
    let about = AboutDialog::new();
    about.set_program_name(Some(PROGRAM_NAME));
    about.set_version(Some(VERSION));
    about.set_copyright(Some(COPYRIGHT));
    about.set_license_type(LICENSE_TYPE);
    about.set_comments(Some(COMMENTS));
    about.set_website(Some(WEBSITE));
    about.set_logo_icon_name(Some(LOGO_ICON_NAME));

    // Disable text selection on all labels in the about dialog
    disable_label_selection(&about);

    about
}

fn disable_label_selection(widget: &impl IsA<gtk::Widget>) {
    if let Some(label) = widget.dynamic_cast_ref::<gtk::Label>() {
        label.set_selectable(false);
        return;
    }
    let mut child = widget.first_child();
    while let Some(c) = child {
        disable_label_selection(&c);
        child = c.next_sibling();
    }
}
