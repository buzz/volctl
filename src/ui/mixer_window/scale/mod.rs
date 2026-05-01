use std::cell::RefCell;
use std::rc::Rc;

use glib::subclass::types::ObjectSubclassIsExt;
use gtk::gio;
use gtk::prelude::{
    AdjustmentExt, ButtonExt, ObjectExt, RangeExt, ScaleExt, SettingsExt, ToggleButtonExt,
    WidgetExt,
};
use gtk::{IconTheme, Image, Orientation, gdk};
use libpulse::volume::{ChannelVolumes, Volume};

use crate::constants::{
    MAX_NATURAL_VOL, MAX_SCALE_VOL, MAX_VOL_SCALE, SETTINGS_ALLOW_EXTRA_VOLUME,
    SETTINGS_SHOW_PERCENTAGE, SETTINGS_VU_ENABLED,
};
use crate::pulse::{MeterData, Pulse, StreamType};

mod imp;

glib::wrapper! {
  pub struct VolumeScale(ObjectSubclass<imp::VolumeScale>)
      @extends gtk::Box, gtk::Widget,
      @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl VolumeScale {
    pub fn new(pulse: Rc<RefCell<Pulse>>, settings: gio::Settings) -> Self {
        let obj: Self = glib::Object::builder()
            .property("orientation", Orientation::Vertical)
            .build();

        let imp = obj.imp();
        imp.pulse.set(pulse.clone()).ok();
        imp.settings.set(settings.clone()).ok();

        imp.allow_extra_volume
            .set(settings.boolean(SETTINGS_ALLOW_EXTRA_VOLUME));

        // Add CSS class for styling
        imp.mute_btn.add_css_class("toggle");

        // Configure scale range based on allow-extra-volume setting
        Self::configure_scale_range(&obj);

        // Clone the Rc reference outside the closure so the imp() borrow is released
        let data: Rc<RefCell<MeterData>> = Rc::clone(&imp.data);
        let scale = imp.scale.clone();

        // Connect slider signal: store handler ID so update() can block it
        {
            let pulse = pulse.clone();
            let data = Rc::clone(&data);
            let handler_id = scale.connect_value_changed(move |scale| {
                let d = data.borrow();
                let index = d.index;
                let stream_type = d.t;
                let ch_count = d.volume.len();
                drop(d);

                // Guard against default MeterData (ch_count == 0) before first update()
                if ch_count == 0 {
                    return;
                }

                let value = scale.value() * MAX_NATURAL_VOL as f64;
                let mut volumes = ChannelVolumes::default();
                volumes.set_len(ch_count);
                volumes.set(ch_count, Volume(value as u32));

                let p = pulse.borrow();
                p.set_volume(stream_type, index, volumes);
            });
            imp.value_changed_handler.set(handler_id).ok();
        }

        // Connect mute button signal: store handler ID so update() can block it
        {
            let pulse = pulse.clone();
            let data = Rc::clone(&data);
            let mute_btn = imp.mute_btn.clone();
            let handler_id = mute_btn.connect_toggled(move |btn| {
                let d = data.borrow();
                let index = d.index;
                let stream_type = d.t;
                drop(d);

                let muted = btn.is_active();
                let p = pulse.borrow();
                p.set_muted(stream_type, index, muted);
            });
            imp.toggled_handler.set(handler_id).ok();
        }

        // React to allow-extra-volume setting changes
        {
            let obj_clone = obj.clone();
            settings.connect_changed(Some(SETTINGS_ALLOW_EXTRA_VOLUME), move |settings, _| {
                let imp = obj_clone.imp();
                imp.allow_extra_volume
                    .set(settings.boolean(SETTINGS_ALLOW_EXTRA_VOLUME));
                Self::configure_scale_range(&obj_clone);
            });
        }

        // React to show-percentage setting changes
        {
            let obj_clone = obj.clone();
            settings.connect_changed(Some(SETTINGS_SHOW_PERCENTAGE), move |settings, _| {
                let show = settings.boolean(SETTINGS_SHOW_PERCENTAGE);
                obj_clone.imp().scale.set_draw_value(show);
            });
        }

        // Set initial draw_value based on show-percentage setting
        let show_percentage = settings.boolean(SETTINGS_SHOW_PERCENTAGE);
        imp.scale.set_draw_value(show_percentage);

        // Configure scale for VU meter appearance if enabled
        let vu_enabled = settings.boolean(SETTINGS_VU_ENABLED);
        imp.vu_enabled.set(vu_enabled);
        // has_origin=false: fill bar extends from the value (knob) position,
        // not from 0. This is needed for correct fill bar rendering.
        imp.scale.set_has_origin(false);
        // Fill bar is always visible: VU shows peaks, non-VU shows volume level.
        imp.scale.set_show_fill_level(true);
        if vu_enabled {
            // Initialize fill_level to 0 BEFORE enabling display.
            imp.scale.set_fill_level(0.0);
        }

        // React to vu-enabled setting changes.
        // Block signals to avoid triggering the volume handler.
        {
            let obj_clone = obj.clone();
            settings.connect_changed(Some(SETTINGS_VU_ENABLED), move |settings, _| {
                let enabled = settings.boolean(SETTINGS_VU_ENABLED);
                let imp = obj_clone.imp();

                // Block the value_changed handler during property changes
                if let Some(id) = imp.value_changed_handler.get() {
                    imp.scale.block_signal(id);
                }

                imp.vu_enabled.set(enabled);
                imp.scale.set_has_origin(false);
                // Fill bar is always visible: VU shows peaks, non-VU shows volume level.
                imp.scale.set_show_fill_level(true);

                // Immediately update fill_level to match the new mode.
                if enabled {
                    // VU mode: fill_level starts at 0, peaks will update it.
                    imp.scale.set_fill_level(0.0);
                } else {
                    // Non-VU mode: fill_level = current volume.
                    imp.scale.set_fill_level(imp.scale.value());
                }

                obj_clone.force_fill_level_redraw();

                if let Some(id) = imp.value_changed_handler.get() {
                    imp.scale.unblock_signal(id);
                }
            });
        }

        obj
    }

    /// Apply scale range, size, and snap-mark based on the current allow-extra-volume setting.
    fn configure_scale_range(obj: &Self) {
        let imp = obj.imp();

        let (upper, height, has_mark) = if imp.allow_extra_volume.get() {
            (MAX_VOL_SCALE, (128.0 * MAX_VOL_SCALE) as i32, true)
        } else {
            (1.0, 128, false)
        };

        // Update adjustment upper bound and clamp current value
        let adj = imp.scale.adjustment();
        adj.set_upper(upper);
        let current = adj.value();
        if current > upper {
            adj.set_value(upper);
        }

        // Update scale height
        imp.scale.set_size_request(24, height);

        // Clear previous marks to prevent duplicate stacking on setting toggles.
        imp.scale.clear_marks();

        // Add snap-mark at 100% when extra volume is enabled.
        if has_mark {
            imp.scale.add_mark(1.0, gtk::PositionType::Left, None);
        }
    }

    pub fn update(&self, data: &MeterData) {
        let imp = self.imp();

        // Collect all values inside a single borrow, then drop it before GTK setters
        let (new_volume, new_muted, new_icon, new_tooltip, needs_icon) = {
            let mut scale_data = imp.data.borrow_mut();
            // Always copy identity fields (type + index) so signal handlers
            // target the correct stream: they default to StreamType::Sink/0.
            scale_data.t = data.t;
            scale_data.index = data.index;

            let icon_changed = data.icon != scale_data.icon;
            if icon_changed {
                scale_data.icon.clone_from(&data.icon);
            }
            if data.name != scale_data.name {
                scale_data.name.clone_from(&data.name);
            }
            let tooltip_changed = data.description != scale_data.description;
            if tooltip_changed {
                scale_data.description.clone_from(&data.description);
            }
            let volume_changed = data.volume != scale_data.volume;
            if volume_changed {
                scale_data.volume = data.volume;
            }
            let mute_changed = data.muted != scale_data.muted;
            if mute_changed {
                scale_data.muted = data.muted;
            }

            (
                if volume_changed {
                    Some(data.volume.avg().0 as f64 / MAX_NATURAL_VOL as f64)
                } else {
                    None
                },
                if mute_changed { Some(data.muted) } else { None },
                if icon_changed {
                    Some(scale_data.icon.clone())
                } else {
                    None
                },
                if tooltip_changed {
                    Some(scale_data.description.clone())
                } else {
                    None
                },
                icon_changed,
            )
        }; // borrow_mut() drops here: RefCell is now free

        // Block GTK signal handlers before calling setters that may fire signals
        if let Some(id) = imp.value_changed_handler.get() {
            imp.scale.block_signal(id);
        }
        if let Some(id) = imp.toggled_handler.get() {
            imp.mute_btn.block_signal(id);
        }

        if needs_icon {
            let icon_name = new_icon.unwrap();
            // Resolve icon name, falling back to default if not found in the theme.
            let icon_name = resolve_icon_name(&icon_name);
            let icon = Image::from_icon_name(&icon_name);
            imp.mute_btn.set_child(Some(&icon));
        }

        if let Some(tooltip) = new_tooltip {
            imp.scale.set_tooltip_text(Some(&tooltip));
            imp.mute_btn.set_tooltip_text(Some(&tooltip));
        }

        if let Some(value) = new_volume {
            imp.scale.set_value(value);
            // In non-VU mode, keep fill_level synced with volume so the fill bar
            // shows the current level. In VU mode, update_peak() manages fill_level.
            if !imp.vu_enabled.get() {
                imp.scale.set_fill_level(value);
            }
        }

        if let Some(muted) = new_muted {
            imp.scale.set_sensitive(!muted);
            imp.mute_btn.set_active(muted);
        }

        // Unblock handlers
        if let Some(id) = imp.value_changed_handler.get() {
            imp.scale.unblock_signal(id);
        }
        if let Some(id) = imp.toggled_handler.get() {
            imp.mute_btn.unblock_signal(id);
        }
    }

    fn format_scale_value(value: f64) -> String {
        format!("{:.0}%", value * 100.0)
    }

    /// Update the VU peak fill level on the scale.
    ///
    /// The `peak` value is in the same units as volume (0..MAX_SCALE_VOL).
    /// For sinks, the peak is scaled by the current volume.
    /// Smooth decay is handled by `Pulse::update()`. This method only displays the current peak value.
    pub fn update_peak(&self, peak: u32, stream_type: StreamType) {
        let imp = self.imp();

        // Normalize peak to 0.0..1.0 range
        let mut normalized = peak as f64 / MAX_SCALE_VOL as f64;

        // For sinks, scale by current volume.
        // This shows the effective output level, not the raw input level.
        if stream_type == StreamType::Sink {
            let data = imp.data.borrow();
            normalized *= data.volume.avg().0 as f64 / MAX_NATURAL_VOL as f64;
            drop(data);
        }

        // Clamp to valid range
        normalized = normalized.clamp(
            0.0,
            if imp.allow_extra_volume.get() {
                1.5
            } else {
                1.0
            },
        );

        // Only update fill_level when VU is enabled.
        if imp.vu_enabled.get() {
            // Block the value_changed handler so GTK setters don't trigger
            // a volume change if they fire value_changed unexpectedly.
            if let Some(id) = imp.value_changed_handler.get() {
                imp.scale.block_signal(id);
            }

            imp.scale.set_fill_level(normalized);

            // Skip repaint if the value hasn't changed meaningfully.
            // This avoids unnecessary redraws when peaks are stable or fully decayed.
            let last = imp.last_displayed_peak.get();
            if (normalized - last).abs() < 0.001 {
                if let Some(id) = imp.value_changed_handler.get() {
                    imp.scale.unblock_signal(id);
                }
                return;
            }
            imp.last_displayed_peak.set(normalized);

            self.force_fill_level_redraw();

            if let Some(id) = imp.value_changed_handler.get() {
                imp.scale.unblock_signal(id);
            }
        }
    }

    /// GTK4 only repaints the fill block when `show_fill_level` changes (bool).
    /// Changing only `fill_level` (float) does NOT trigger a repaint.
    fn force_fill_level_redraw(&self) {
        let imp = self.imp();
        imp.scale.set_show_fill_level(false);
        imp.scale.set_show_fill_level(true);
    }
}

/// Check if an icon name exists in the default GTK icon theme, falling back to
/// `"multimedia-volume-control"` if not found.
fn resolve_icon_name(icon_name: &str) -> String {
    if let Some(display) = gdk::Display::default() {
        let theme = IconTheme::for_display(&display);
        if theme.has_icon(icon_name) {
            return icon_name.to_owned();
        }
    }
    "multimedia-volume-control".to_owned()
}
