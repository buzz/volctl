use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct OsdStateController {
    volume: Cell<u32>,
    muted: Cell<bool>,
    opacity: Rc<Cell<f64>>,
    hide_timeout_id: Rc<RefCell<Option<glib::SourceId>>>,
    fade_timeout_id: Rc<RefCell<Option<glib::SourceId>>>,
}

impl OsdStateController {
    pub fn new() -> Self {
        Self {
            volume: Cell::new(0),
            muted: Cell::new(false),
            opacity: Rc::new(Cell::new(1.0)),
            hide_timeout_id: Rc::new(RefCell::new(None)),
            fade_timeout_id: Rc::new(RefCell::new(None)),
        }
    }

    /// Update volume and muted state, reset opacity for fresh display
    pub fn update_volume(&self, volume: u32, muted: bool) {
        self.volume.set(volume);
        self.muted.set(muted);
        self.reset_opacity();
    }

    /// Reset opacity to fully visible and cancel any active fade
    pub fn reset_opacity(&self) {
        self.opacity.set(1.0);
    }

    /// Set opacity (used during fade animation)
    pub fn set_opacity(&self, opacity: f64) {
        self.opacity.set(opacity);
    }

    /// Begin hide timeout - starts fade when timeout expires
    pub fn begin_hide_timeout<F>(&self, timeout_ms: u32, on_hide: F)
    where
        F: Fn() + 'static,
    {
        self.cancel_hide_timeout();

        let hide_id_ref = self.hide_timeout_id.clone();

        let source_id = glib::timeout_add_local(
            std::time::Duration::from_millis(timeout_ms as u64),
            move || {
                // Clear before breaking: prevents cancel_timers() from calling
                // remove() on a source GLib has already destroyed.
                *hide_id_ref.borrow_mut() = None;

                on_hide();
                glib::ControlFlow::Break
            },
        );

        *self.hide_timeout_id.borrow_mut() = Some(source_id);
    }

    /// Begin fade-out animation
    pub fn begin_fade_out<F>(&self, on_opacity_change: F)
    where
        F: Fn(f64) + 'static,
    {
        self.cancel_fade_timeout();

        let opacity = self.opacity.clone();
        let fade_id_ref = self.fade_timeout_id.clone();

        let source_id = glib::timeout_add_local(std::time::Duration::from_millis(30), move || {
            let current = opacity.get();
            let new_opacity = (current - 0.05).max(0.0);

            opacity.set(new_opacity);
            on_opacity_change(new_opacity);

            if new_opacity > 0.0 {
                glib::ControlFlow::Continue
            } else {
                // Fade complete: clear before breaking so cancel_timers() won't
                // try to remove an already-gone source.
                *fade_id_ref.borrow_mut() = None;
                glib::ControlFlow::Break
            }
        });

        *self.fade_timeout_id.borrow_mut() = Some(source_id);
    }

    /// Cancel hide timeout if active
    pub fn cancel_hide_timeout(&self) {
        let id = self.hide_timeout_id.borrow_mut().take();
        // Release the borrow before `remove()` so that even a synchronous
        // callback firing cannot re-borrow the RefCell.
        if let Some(id) = id {
            id.remove();
        }
    }

    /// Cancel fade timeout if active
    pub fn cancel_fade_timeout(&self) {
        let id = self.fade_timeout_id.borrow_mut().take();
        // Release the borrow before `remove()` so that even a synchronous
        // callback firing cannot re-borrow the RefCell.
        if let Some(id) = id {
            id.remove();
        }
    }

    /// Cancel all timers
    pub fn cancel_timers(&self) {
        self.cancel_hide_timeout();
        self.cancel_fade_timeout();
    }

    /// Get current volume (0-65536)
    pub fn get_volume(&self) -> u32 {
        self.volume.get()
    }

    /// Get muted state
    pub fn get_muted(&self) -> bool {
        self.muted.get()
    }

    /// Get current opacity (0.0-1.0)
    pub fn get_opacity(&self) -> f64 {
        self.opacity.get()
    }

    /// Get normalized volume (0.0-1.0)
    pub fn get_volume_normalized(&self) -> f64 {
        self.volume.get() as f64 / crate::constants::MAX_NATURAL_VOL as f64
    }
}
