use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gtk::gio;
use tracing;

use crate::pulse::Pulse;

use gdk::prelude::{DeviceExt, DisplayExt, ListModelExt, MonitorExt, SeatExt};
use glib::object::Cast;
use glib::subclass::types::ObjectSubclassIsExt;
use glib::translate::ToGlibPtr;
use gtk::prelude::{BoxExt, WidgetExt};

use super::utils::{DisplayType, get_display_type};
use super::x11::X11Context;
use crate::pulse::StreamData;
use scale::VolumeScale;

mod imp;
mod scale;
mod wayland;
mod x11;

glib::wrapper! {
  pub struct MixerWindow(ObjectSubclass<imp::MixerWindow>)
      @extends gtk::Window, gtk::Widget,
      @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl MixerWindow {
    pub fn new(
        pulse: Rc<RefCell<Pulse>>,
        settings: gio::Settings,
        x11_context: Option<X11Context>,
        parent_ref: Rc<RefCell<Option<MixerWindow>>>,
    ) -> Self {
        let window: MixerWindow = glib::Object::builder()
            .property("decorated", false)
            .property("resizable", false)
            .property("deletable", false)
            .build();

        let imp = window.imp();
        imp.pulse.set(pulse).ok();
        imp.settings.set(settings).ok();
        if let Some(ctx) = x11_context {
            *imp.x11_context.borrow_mut() = Some(ctx);
        }
        *imp.parent_ref.borrow_mut() = Some(parent_ref.clone());

        // Start auto-close timeout after settings is set
        imp.enable_auto_close_timeout();

        window
    }

    pub fn update_sinks(&self, sink_streams: &HashMap<u32, StreamData>) {
        self.update_volume_scales(sink_streams, self.imp().sinks.clone());
        self.update_separator_visibility();
    }

    pub fn update_sink_inputs(&self, sink_input_streams: &HashMap<u32, StreamData>) {
        self.update_volume_scales(sink_input_streams, self.imp().sink_inputs.clone());
        self.update_separator_visibility();
    }

    /// Update separator visibility based on whether we have both sinks and sink inputs
    fn update_separator_visibility(&self) {
        let imp = self.imp();
        let sinks = imp.sinks.borrow();
        let sink_inputs = imp.sink_inputs.borrow();
        let mut separator = imp.separator.borrow_mut();

        // Show separator only if we have both sinks and sink inputs
        if !sinks.is_empty() && !sink_inputs.is_empty() {
            if separator.is_none() {
                let sep = gtk::Separator::new(gtk::Orientation::Vertical);
                sep.set_margin_top(8);
                let box_ = imp.box_.borrow();
                // Find the last sink widget by iterating through box children
                let mut last_sink: Option<gtk::Widget> = None;
                let children = box_.observe_children();
                for i in 0..children.n_items() {
                    if let Some(child) = children
                        .item(i)
                        .and_then(|o| o.downcast::<gtk::Widget>().ok())
                    {
                        let child_obj: glib::Object = child.clone().into();
                        if sinks.values().any(|scale| {
                            let scale_obj: glib::Object = scale.clone().into();
                            scale_obj == child_obj
                        }) {
                            last_sink = Some(child);
                        }
                    }
                }
                if let Some(ref last) = last_sink {
                    box_.insert_child_after(&sep, Some(last));
                } else {
                    box_.append(&sep);
                }
                *separator = Some(sep);
            }
        } else if let Some(sep) = separator.take() {
            let box_ = imp.box_.borrow();
            box_.remove(&sep);
        }
    }

    fn update_volume_scales(
        &self,
        streams: &HashMap<u32, StreamData>,
        scales: Rc<RefCell<HashMap<u32, VolumeScale>>>,
    ) {
        let imp = self.imp();
        let box_ = imp.box_.borrow();
        let mut scales = scales.borrow_mut();

        // Remove scales?
        scales.retain(|stream_idx, scale| {
            let keep = streams.contains_key(stream_idx);
            if !keep {
                box_.remove(scale);
            }
            keep
        });

        for (stream_idx, stream) in streams {
            // Add or get sink from hashmap
            let pulse_rc = imp.pulse.get().unwrap().clone();
            let settings = imp.settings.get().unwrap().clone();
            let scale = scales
                .entry(*stream_idx)
                .or_insert_with(|| VolumeScale::new(pulse_rc.clone(), settings.clone()));

            // Append sink widget
            if scale.parent().is_none() {
                box_.append(scale);
            }

            scale.update(&stream.data);
        }
    }

    pub fn move_(&self, x: i32, y: i32) {
        match get_display_type() {
            Ok(DisplayType::Wayland) => self.move_wayland(x, y),
            Ok(DisplayType::X11) => self.move_x11(x, y),
            Err(e) => {
                tracing::warn!(error = %e, "Failed to detect display type for mixer window");
            }
        }
    }
}

/// Calculate the mixer window position using screen-quadrant logic.
///
/// Given the tray-click anchor `(x, y)` and the window's allocated size, decides
/// which side of the pointer the window should appear so that it stays on-screen
/// and doesn't overlap the taskbar.
pub(crate) fn calculate_mixer_position(window: &MixerWindow, x: i32, y: i32) -> (i32, i32) {
    let (min_size, _natural) = window.preferred_size();
    // Requisition is a FFI struct; access fields via the raw pointer.
    let (win_w, win_h) = unsafe {
        let ptr = min_size.to_glib_none().0;
        ((*ptr).width, (*ptr).height)
    };

    let (x, y) = if x == 0 && y == 0 {
        if let Some((_, px, py)) = get_pointer_position() {
            (px as i32, py as i32)
        } else {
            (x, y)
        }
    } else {
        (x, y)
    };

    // Find the monitor that contains the click point.
    let monitor_rect = find_monitor_at_point(x, y);

    // Apply quadrant logic:
    //   - anchor is in left  half of monitor → window goes to the RIGHT  of anchor
    //   - anchor is in right half of monitor → window goes to the LEFT   of anchor
    //   - anchor is in top    half of monitor → window goes BELOW  anchor
    //   - anchor is in bottom half of monitor → window goes ABOVE  anchor
    let win_x = if (x - monitor_rect.x()) < monitor_rect.width() / 2 {
        x
    } else {
        x - win_w
    };

    let win_y = if (y - monitor_rect.y()) < monitor_rect.height() / 2 {
        y
    } else {
        y - win_h
    };

    // Clamp so the window never overflows the right or bottom monitor edge.
    let mut final_x = win_x;
    let mut final_y = win_y;

    if final_x + win_w > monitor_rect.x() + monitor_rect.width() {
        final_x = monitor_rect.x() + monitor_rect.width() - win_w;
    }
    if final_y + win_h > monitor_rect.y() + monitor_rect.height() {
        final_y = monitor_rect.y() + monitor_rect.height() - win_h;
    }

    // Also ensure we don't go off the left or top edge.
    if final_x < monitor_rect.x() {
        final_x = monitor_rect.x();
    }
    if final_y < monitor_rect.y() {
        final_y = monitor_rect.y();
    }

    (final_x, final_y)
}

/// Get the current mouse pointer position.
///
/// Returns `(monitor, x, y)` or `None` if the seat/pointer is unavailable.
fn get_pointer_position() -> Option<(gdk::Monitor, f64, f64)> {
    let display = gdk::Display::default()?;
    let seat = display.default_seat()?;
    let pointer = seat.pointer()?;
    let (surface, x, y) = pointer.surface_at_position();
    let monitor = surface
        .and_then(|s| display.monitor_at_surface(&s))
        .or_else(|| {
            display
                .monitors()
                .item(0)
                .and_then(|o: glib::Object| o.downcast::<gdk::Monitor>().ok())
        })?;
    Some((monitor, x, y))
}

/// Find the monitor whose geometry contains the given point.
///
/// Falls back to the primary (first) monitor if no monitor contains the point.
fn find_monitor_at_point(x: i32, y: i32) -> gdk::Rectangle {
    let display = gdk::Display::default();
    let display = match display {
        Some(d) => d,
        None => return gdk::Rectangle::new(0, 0, 1, 1),
    };

    let monitors = display.monitors();
    for i in 0..monitors.n_items() {
        let Some(obj) = monitors.item(i) else {
            continue;
        };
        let Ok(monitor) = obj.downcast::<gdk::Monitor>() else {
            continue;
        };
        let geo = monitor.geometry();
        if x >= geo.x() && x < geo.x() + geo.width() && y >= geo.y() && y < geo.y() + geo.height() {
            return geo;
        }
    }

    // Fallback: primary monitor
    if let Some(obj) = monitors.item(0)
        && let Ok(monitor) = obj.downcast::<gdk::Monitor>()
    {
        return monitor.geometry();
    }

    gdk::Rectangle::new(0, 0, 1, 1)
}
