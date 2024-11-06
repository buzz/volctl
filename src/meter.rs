use libpulse::volume::ChannelVolumes;

/** The maximum natural volume, i.e. 100% */
pub const MAX_NATURAL_VOL: u32 = 65536;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StreamType {
    Sink,
    SinkInput,
    Source,
    SourceOutput,
}

impl Default for StreamType {
    fn default() -> Self {
        StreamType::Sink
    }
}

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
