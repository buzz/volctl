pub mod wayland;
pub mod x11;

/// Trait for OSD surface backends (X11, Wayland, etc.)
pub trait SurfaceBackend {
    /// Show the OSD window
    fn show(&self);
    /// Update OSD position
    fn update_position(&self, position: &str);
    /// Update OSD scale (triggers resize)
    fn update_scale(&self, scale: f64);
    /// Begin fade-out animation with given opacity
    fn begin_fade_out(&self, opacity: f64);
    /// Check if a compositor is available (for fade-out vs immediate hide)
    fn is_composited(&self) -> bool;
    /// Destroy the surface and release all resources (window, connection, timers)
    fn destroy(&self);
}
