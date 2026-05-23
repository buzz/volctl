use std::cell::RefCell;
use std::rc::Rc;

use libpulse::stream::Stream;
use libpulse::volume::ChannelVolumes;

use crate::constants::{MAX_NATURAL_VOL, MAX_SCALE_VOL, MAX_VOL_SCALE, PEAKS_RATE};

/// Type of PulseAudio stream.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum StreamType {
    #[default]
    Sink,
    SinkInput,
}

/// Meter data for a single stream (sink or sink input).
#[derive(Debug, Clone, Default)]
pub struct MeterData {
    pub t: StreamType,
    pub index: u32,

    pub name: String,
    pub icon: String,
    pub description: String,

    pub volume: ChannelVolumes,
    pub muted: bool,
}

/// Transferrable information pertaining to a stream.
#[derive(Debug)]
pub struct TxStreamData {
    pub data: MeterData,
    pub monitor_index: u32,
}

/// Stored representation of a pulse stream.
pub struct StreamData {
    pub data: MeterData,
    pub peak: u32,
    /// Monotonic time (ms) when `peak` was last updated. Used for smooth decay.
    pub peak_time: u64,
    pub monitor_index: u32,
    /// Monitor stream connection, kept alive by this Rc.
    pub monitor: Rc<RefCell<Stream>>,
}

// --- Peak helpers ---

/// Apply a minimum decay floor to a new peak value.
///
/// Prevents the peak from dropping by more than one sample period's worth of decay
/// between consecutive peak samples. Matches pavucontrol's `updatePeak` behavior
/// for smoother VU meter transitions.
pub fn apply_peak_floor(current_peak: u32, new_peak: u32) -> u32 {
    let floor = MAX_SCALE_VOL as f64 / PEAKS_RATE as f64;
    if current_peak as f64 >= floor {
        let min_val = (current_peak as f64 - floor).round() as u32;
        new_peak.max(min_val)
    } else {
        new_peak
    }
}

/// Helper function to scale a raw peak value (0..1) to the volume scale (0..MAX_SCALE_VOL).
///
/// `PA_STREAM_PEAK_DETECT` writes the peak amplitude directly to the buffer as a float in [0, 1].
/// We scale it to match the volume scale used by the VU meter.
pub fn calculate_peak(raw_peak: f32) -> u32 {
    (raw_peak * MAX_NATURAL_VOL as f32 * MAX_VOL_SCALE as f32).round() as u32
}

// --- Metadata helpers ---

/// Resolve the icon name for a sink input, following the same fallback chain as the Python
/// implementation (`slider_win._icon_name_from_sink_input`).
///
/// Fallback order:
/// 1. `application.icon_name` property
/// 2. `media.icon_name` property
/// 3. `application.process.binary` property (e.g., "firefox")
/// 4. `application.name` lowercased with spaces replaced by dashes
/// 5. If no icon name found at all → `"multimedia-volume-control"`
pub fn get_icon_name_from_sink_input(proplist: &libpulse::proplist::Proplist) -> String {
    proplist
        .get_str("application.icon_name")
        .or_else(|| proplist.get_str("media.icon_name"))
        .or_else(|| proplist.get_str("application.process.binary"))
        .or_else(|| {
            proplist
                .get_str("application.name")
                .map(|name| name.to_lowercase().replace(' ', "-"))
        })
        .map(|s| s.to_owned())
        .unwrap_or_else(|| "multimedia-volume-control".to_owned())
}

/// Build the description tooltip for a sink input, following the same format as the Python
/// implementation (`slider_win._name_from_sink_input`).
///
/// Format: `<b>Application Name</b>: Media Name` (e.g., `<b>mpv</b>: Song Title - Artist`)
/// Falls back to just `application.name`, then to `sink_input.name`.
pub fn get_description_from_sink_input(
    item: &libpulse::context::introspect::SinkInputInfo<'_>,
) -> String {
    let app_name = item.proplist.get_str("application.name");
    let media_name = item.proplist.get_str("media.name");

    match (app_name, media_name) {
        (Some(app), Some(media)) => format!("<b>{}</b>: {}", app, media),
        (Some(app), None) => app.to_owned(),
        (None, _) => item.name.clone().unwrap_or_default().into_owned(),
    }
}
