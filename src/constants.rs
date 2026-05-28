pub const APP_ID: &str = "org.volctl";

/// Default external mixer command
pub const DEFAULT_MIXER_CMD: &str = "pavucontrol";

/// The maximum natural volume, i.e. 100%
pub const MAX_NATURAL_VOL: u32 = 65536;

/// The maximum volume scale, i.e. 150%
pub const MAX_VOL_SCALE: f64 = 1.5;

/// The maximum scale volume
pub const MAX_SCALE_VOL: u32 = (MAX_NATURAL_VOL as f64 * MAX_VOL_SCALE) as u32;

/// Peak monitor rate
pub const PEAKS_RATE: u32 = 144;

// Settings
pub const SETTINGS_SCHEMA_KEY: &str = "apps.volctl";
pub const SETTINGS_PATH: &str = "/apps/volctl/";

// Setting keys
pub const SETTINGS_USE_SYMBOLIC_ICONS: &str = "use-symbolic-icons";
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
pub const SETTINGS_USE_LAYER_SHELL: &str = "use-layer-shell";
pub const SETTINGS_MIXER_POSITION: &str = "mixer-position";
pub const SETTINGS_OSD_MARGIN: &str = "osd-margin";
pub const SETTINGS_OSD_FADE_ENABLED: &str = "osd-fade-enabled";
pub const SETTINGS_MIXER_MARGIN: &str = "mixer-margin";

// OSD constants
pub const OSD_DEFAULT_TIMEOUT: u32 = 1000;
pub const OSD_BASE_WIDTH: f64 = 200.0;
pub const OSD_BASE_HEIGHT: f64 = 200.0;
pub const OSD_BASE_FONT_SIZE: f64 = 32.0;
pub const OSD_BASE_LINE_WIDTH: f64 = 5.0;
pub const OSD_BASE_PADDING: f64 = 24.0;
pub const OSD_BG_OPACITY: f64 = 0.85;
pub const OSD_BG_CORNER_RADIUS: f64 = 8.0;
pub const OSD_MUTE_OPACITY: f64 = 0.2;
pub const OSD_TEXT_OPACITY: f64 = 0.8;
pub const OSD_NUM_BARS: i32 = 16;
