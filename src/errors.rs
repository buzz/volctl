use thiserror::Error;

#[derive(Error, Debug)]
pub enum PulseError {
    #[error("Failed to initialize PulseAudio mainloop")]
    MainloopInit,
    #[error("Failed to initialize PulseAudio context")]
    ContextInit,
    #[error("Failed to start PulseAudio mainloop")]
    MainloopStart,
    #[error("PulseAudio connection failed: {0}")]
    ConnectionFailed(String),
    #[error("PulseAudio session terminated")]
    SessionTerminated,
    #[error("Failed to create monitor stream")]
    StreamCreation,
}

#[derive(Error, Debug)]
pub enum X11Error {
    #[error("Failed to get GDK display")]
    NoDisplay,
    #[error("GDK display is not X11")]
    NotX11Display,
}
