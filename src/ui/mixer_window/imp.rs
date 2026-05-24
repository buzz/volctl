use std::cell::{Cell, OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use glib::object::ObjectExt;
use glib::subclass::object::{ObjectImpl, ObjectImplExt};
use glib::subclass::types::{ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt};
use gtk::gio;
use gtk::prelude::{EventControllerExt, GtkWindowExt, SettingsExt, WidgetExt};
use gtk::subclass::widget::WidgetImplExt;
use gtk::subclass::{widget::WidgetImpl, window::WindowImpl};
use gtk::{
    Box, EventControllerKey, EventControllerMotion, Orientation, PropagationPhase, Separator,
};

use crate::constants::{SETTINGS_AUTO_CLOSE, SETTINGS_TIMEOUT};
use crate::pulse::Pulse;
use crate::ui::utils::{DisplayType, get_display_type};
use crate::ui::x11::X11Context;

use super::scale::VolumeScale;

const COL_SPACING: i32 = 2;
const PADDING: i32 = 8;

pub struct MixerWindow {
    pub(super) box_: Rc<RefCell<Box>>,
    // Separator between audio interface and application sliders
    pub(super) separator: Rc<RefCell<Option<Separator>>>,
    // Stores scale widgets by stream index
    pub(super) sinks: Rc<RefCell<HashMap<u32, VolumeScale>>>,
    pub(super) sink_inputs: Rc<RefCell<HashMap<u32, VolumeScale>>>,
    pub(super) x11_context: RefCell<Option<X11Context>>,
    pub(super) pulse: OnceCell<Rc<RefCell<Pulse>>>,
    pub(super) settings: OnceCell<gio::Settings>,
    pub(super) auto_close_timeout: RefCell<Option<glib::SourceId>>,
    // Back-reference to the app's mixer_window RefCell so we can clear ourselves
    pub(super) parent_ref: RefCell<Option<Rc<RefCell<Option<super::MixerWindow>>>>>,
    // Whether the X11 window position has been applied (prevents double-positioning)
    pub(super) position_applied: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for MixerWindow {
    const NAME: &'static str = "VolctlMixerWindow";
    type Type = super::MixerWindow;
    type ParentType = gtk::Window;
}

impl ObjectImpl for MixerWindow {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        obj.set_child(Some(&*self.box_.borrow()));
        obj.set_visible(false);
        // Make the window accept keyboard focus so key events (e.g., Escape) work
        obj.set_focus(Some(&*self.box_.borrow()));

        // Add event controller for mouse enter/leave to reset auto-close timeout
        let controller = EventControllerMotion::new();
        let weak_obj = obj.downgrade();
        let weak_obj_leave = weak_obj.clone();
        controller.connect_enter(move |_ctrl, _x, _y| {
            if let Some(win) = weak_obj.upgrade() {
                win.imp().cancel_auto_close_timeout();
            }
        });
        controller.connect_leave(move |_ctrl| {
            if let Some(win) = weak_obj_leave.upgrade() {
                win.imp().enable_auto_close_timeout();
            }
        });
        obj.add_controller(controller);

        // Add event controller for key press to close on Escape
        // Use capture phase so it fires before child widgets handle the key
        let key_controller = EventControllerKey::new();
        key_controller.set_propagation_phase(PropagationPhase::Capture);
        let weak_obj_key = obj.downgrade();
        let parent_ref_key = self.parent_ref.borrow().clone();
        key_controller.connect_key_pressed(move |_ctrl, key, _code, _state| {
            if key == gdk::Key::Escape {
                if let Some(win) = weak_obj_key.upgrade() {
                    win.imp().cancel_auto_close_timeout();
                    win.destroy();
                    if let Some(ref_) = &parent_ref_key {
                        *ref_.borrow_mut() = None;
                    }
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        obj.add_controller(key_controller);
    }
}
impl MixerWindow {
    pub fn enable_auto_close_timeout(&self) {
        // Cancel any existing timeout first
        self.cancel_auto_close_timeout();

        if let Some(settings) = self.settings.get()
            && settings.boolean(SETTINGS_AUTO_CLOSE)
        {
            let timeout_ms = settings.int(SETTINGS_TIMEOUT) as u32;
            let weak_obj = self.obj().downgrade();
            let parent_ref = self.parent_ref.borrow().clone();
            let id = glib::timeout_add_local(
                std::time::Duration::from_millis(timeout_ms as u64),
                move || {
                    // Defer destruction to the next event cycle so GTK can fully
                    // process the current event before destroying the window.
                    // This prevents the first click after auto-close from being swallowed.
                    // Clear imp.mixer_window AFTER destroy() so there's no window
                    // where a new click could spawn a duplicate mixer window.
                    if let Some(win) = weak_obj.upgrade() {
                        let weak_win = win.downgrade();
                        let parent_ref_clone = parent_ref.clone();
                        glib::timeout_add_local(std::time::Duration::ZERO, move || {
                            if let Some(w) = weak_win.upgrade() {
                                w.destroy();
                            }
                            if let Some(ref_) = &parent_ref_clone {
                                *ref_.borrow_mut() = None;
                            }
                            glib::ControlFlow::Break
                        });
                    }
                    glib::ControlFlow::Break
                },
            );
            *self.auto_close_timeout.borrow_mut() = Some(id);
        }
    }

    pub fn cancel_auto_close_timeout(&self) {
        if let Some(id) = self.auto_close_timeout.borrow_mut().take() {
            id.remove();
        }
    }
}

impl WindowImpl for MixerWindow {}

impl WidgetImpl for MixerWindow {
    fn realize(&self) {
        self.parent_realize();

        if matches!(get_display_type(), Ok(DisplayType::X11)) {
            self.obj().realize_x11();
        }
    }
}

impl Default for MixerWindow {
    fn default() -> Self {
        Self {
            box_: Rc::from(RefCell::from(
                Box::builder()
                    .orientation(Orientation::Horizontal)
                    .spacing(COL_SPACING)
                    .margin_top(PADDING)
                    .margin_bottom(PADDING)
                    .margin_start(PADDING)
                    .margin_end(PADDING)
                    .build(),
            )),
            separator: Rc::from(RefCell::from(None)),
            sinks: Rc::from(RefCell::from(HashMap::new())),
            sink_inputs: Rc::from(RefCell::from(HashMap::new())),
            x11_context: RefCell::from(None),
            pulse: OnceCell::new(),
            settings: OnceCell::new(),
            auto_close_timeout: RefCell::from(None),
            parent_ref: RefCell::from(None),
            position_applied: Cell::new(false),
        }
    }
}
