pub const APP_ID: &str = "org.volctl";

/// The maximum natural volume, i.e. 100%
pub const MAX_NATURAL_VOL: u32 = 65536;

/// The maximum scale volume, i.e. 150%
pub const MAX_SCALE_VOL: u32 = (MAX_NATURAL_VOL as f64 * 1.5) as u32;

// Setting keys
pub const SETTINGS_MOUSE_WHEEL_STEP: &str = "mouse-wheel-step";
pub const SETTINGS_MIXER_COMMAND: &str = "mixer-command";
pub const SETTINGS_SHOW_PERCENTAGE: &str = "show-percentage";
pub const SETTINGS_VU_ENABLED: &str = "vu-enabled";
pub const SETTINGS_AUTO_CLOSE: &str = "auto-close";
pub const SETTINGS_TIMEOUT: &str = "timeout";
pub const SETTINGS_ALLOW_EXTRA_VOLUME: &str = "allow-extra-volume";
pub const SETTINGS_OSD_ENABLED: &str = "osd-enabled";
pub const SETTINGS_OSD_TIMEOUT: &str = "osd-timeout";
pub const SETTINGS_OSD_SCALE: &str = "osd-scale";
pub const SETTINGS_OSD_POSITION: &str = "osd-position";
