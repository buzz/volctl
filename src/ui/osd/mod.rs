pub mod controller;
pub mod render;
pub mod surface;
pub mod widget;

use std::cell::Cell;
use std::rc::Rc;

use gtk::gio::Settings;
use gtk::prelude::*;

use crate::constants::{
    OSD_DEFAULT_TIMEOUT, SETTINGS_OSD_ENABLED, SETTINGS_OSD_POSITION, SETTINGS_OSD_SCALE,
    SETTINGS_OSD_TIMEOUT,
};
use crate::ui::osd::controller::OsdStateController;
use crate::ui::osd::surface::SurfaceBackend;
use crate::ui::osd::surface::wayland::WaylandSurface;
use crate::ui::osd::surface::x11::X11Surface;
use crate::ui::utils::{DisplayType, Position};
use crate::ui::x11::X11Context;

/// Cached OSD settings, refreshed via gsettings `changed` signals.
#[derive(Clone)]
struct OsdSettingsCache {
    enabled: Rc<Cell<bool>>,
    scale: Rc<Cell<f64>>,
    position: Rc<Cell<Position>>,
    timeout: Rc<Cell<u32>>,
}

impl OsdSettingsCache {
    fn new(settings: &Settings) -> Self {
        let enabled = Rc::new(Cell::new(settings.boolean(SETTINGS_OSD_ENABLED)));
        let scale = Rc::new(Cell::new(settings.int(SETTINGS_OSD_SCALE) as f64 / 100.0));
        let position = Rc::new(Cell::new(
            Position::try_from(settings.enum_(SETTINGS_OSD_POSITION))
                .expect("invalid osd-position value"),
        ));
        let timeout = Rc::new(Cell::new(if settings.int(SETTINGS_OSD_TIMEOUT) == 0 {
            OSD_DEFAULT_TIMEOUT
        } else {
            settings.int(SETTINGS_OSD_TIMEOUT) as u32
        }));

        // Listen for changes and refresh cache
        {
            let enabled = enabled.clone();
            settings.connect_changed(Some(SETTINGS_OSD_ENABLED), move |s, _| {
                enabled.set(s.boolean(SETTINGS_OSD_ENABLED));
            });
        }
        {
            let scale = scale.clone();
            settings.connect_changed(Some(SETTINGS_OSD_SCALE), move |s, _| {
                scale.set(s.int(SETTINGS_OSD_SCALE) as f64 / 100.0);
            });
        }
        {
            let position = position.clone();
            settings.connect_changed(Some(SETTINGS_OSD_POSITION), move |s, _| {
                position.set(
                    Position::try_from(s.enum_(SETTINGS_OSD_POSITION))
                        .expect("invalid osd-position value"),
                );
            });
        }
        {
            let timeout = timeout.clone();
            settings.connect_changed(Some(SETTINGS_OSD_TIMEOUT), move |s, _| {
                let val = if s.int(SETTINGS_OSD_TIMEOUT) == 0 {
                    OSD_DEFAULT_TIMEOUT
                } else {
                    s.int(SETTINGS_OSD_TIMEOUT) as u32
                };
                timeout.set(val);
            });
        }

        Self {
            enabled,
            scale,
            position,
            timeout,
        }
    }
}

pub struct OsdController {
    surface: Rc<dyn SurfaceBackend>,
    controller: Rc<OsdStateController>,
    settings: OsdSettingsCache,
    first_update: Cell<bool>,
}

impl OsdController {
    pub fn new(
        settings: &Settings,
        x11_context: Option<X11Context>,
        display_type: DisplayType,
        application: &gtk::Application,
    ) -> Self {
        let controller = Rc::new(OsdStateController::new());

        let surface: Rc<dyn SurfaceBackend> = match display_type {
            DisplayType::X11 => {
                // Safe: caller guarantees x11_context is Some when display_type is X11
                let ctx = x11_context.expect("X11 context required on X11 display");
                Rc::new(X11Surface::new(
                    settings,
                    controller.clone(),
                    ctx,
                    application,
                ))
            }
            DisplayType::Wayland => Rc::new(WaylandSurface::new(
                settings,
                controller.clone(),
                application,
            )),
        };

        Self {
            surface,
            controller,
            settings: OsdSettingsCache::new(settings),
            first_update: Cell::new(true),
        }
    }

    /// Update OSD with new volume state
    pub fn update(&self, volume: u32, muted: bool) {
        // Skip first update (startup volume read)
        if self.first_update.get() {
            self.first_update.set(false);
            return;
        }

        // Check if OSD is enabled
        if !self.settings.enabled.get() {
            return;
        }

        // Only update if volume changed
        if volume == self.controller.get_volume() && muted == self.controller.get_muted() {
            return;
        }

        // Cancel any pending hide/fade
        self.controller.cancel_timers();

        // Update controller state
        self.controller.update_volume(volume, muted);

        // Read cached settings (refreshed via gsettings `changed` signals)
        let scale = self.settings.scale.get();
        let position = self.settings.position.get();
        let timeout = self.settings.timeout.get();

        // Update surface position and scale
        self.surface.update_position(position);
        self.surface.update_scale(scale);

        // Show OSD
        self.surface.show();

        // Clone Rc references for closures (surfaces share the same Rc)
        let controller_rc = self.controller.clone();
        let surface_rc = self.surface.clone();

        // Start hide timeout
        let c_hide = controller_rc.clone();
        c_hide.begin_hide_timeout(timeout, move || {
            let s_check = surface_rc.clone();
            if s_check.is_composited() {
                // Start fade-out animation when compositor is available
                let surf = surface_rc.clone();
                controller_rc.begin_fade_out(move |opacity| {
                    surf.begin_fade_out(opacity);
                });
            } else {
                // No compositor: immediate hide without fade-out
                controller_rc.set_opacity(0.0);
                surface_rc.begin_fade_out(0.0);
            }
        });
    }

    /// Destroy the OSD and release all resources
    pub fn destroy(&self) {
        self.controller.cancel_timers();
        self.surface.destroy();
    }
}
