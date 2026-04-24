use gtk::{AboutDialog, License};

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
    about
}
