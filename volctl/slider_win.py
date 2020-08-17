"""
VolumeSliders window


Small window that appears next to tray icon when activated. It displays
master and app volume sliders.
"""

from gi.repository import Gtk, Gdk, GLib, GObject

from volctl.lib.pulseaudio import (
    PA_VOLUME_MUTED,
    PA_VOLUME_NORM,
    pa_threaded_mainloop_lock,
    pa_threaded_mainloop_unlock,
)


class VolumeSliders(Gtk.Window):
    """Window that displays volume sliders."""

    SPACING = 6

    def __init__(self, volctl, monitor_rect):
        super().__init__(type=Gtk.WindowType.POPUP)
        self._volctl = volctl
        self._monitor_rect = monitor_rect
        self._grid = None
        self._show_percentage = self._volctl.settings.get_boolean("show-percentage")

        # gui objects by index
        self._sink_scales = None
        self._sink_input_scales = None

        self.connect("enter-notify-event", self._cb_enter_notify)
        self.connect("leave-notify-event", self._cb_leave_notify)

        self._frame = Gtk.Frame()
        self._frame.set_shadow_type(Gtk.ShadowType.OUT)
        self.add(self._frame)
        self.create_sliders()

        # timeout
        self._timeout = None
        self._enable_timeout()

    def set_increments(self):
        """Set sliders increment step."""
        for _, scale in self._sink_scales.items():
            self._set_increments_on_scale(scale)
        for _, scale in self._sink_input_scales.items():
            self._set_increments_on_scale(scale)

    def reset_timeout(self):
        """Reset auto-close timeout."""
        self._remove_timeout()
        self._enable_timeout()

    def _set_increments_on_scale(self, scale):
        scale.set_increments(
            PA_VOLUME_NORM / self._volctl.mouse_wheel_step,
            PA_VOLUME_NORM / self._volctl.mouse_wheel_step,
        )

    def _set_position(self):
        status_icon = self._volctl.tray_icon
        info_avail, screen, tray_rect, orient = status_icon.get_geometry()
        if not info_avail:
            raise ValueError("StatusIcon position information not available!")
        win_w, win_h = self.get_size()

        # initial position (window anchor based on screen quadrant)
        win_x = tray_rect.x
        win_y = tray_rect.y
        if tray_rect.x < self._monitor_rect.width / 2:
            win_x += tray_rect.width
        else:
            if orient == Gtk.Orientation.VERTICAL:
                win_x -= win_w
        if tray_rect.y < self._monitor_rect.height / 2:
            win_y += tray_rect.height
        else:
            win_y -= win_h

        # keep window inside screen
        if win_x + win_w > self._monitor_rect.width:
            win_x = self._monitor_rect.width - win_w

        self.set_screen(screen)
        self.move(win_x, win_y)

    def create_sliders(self):
        """(Re-)create sliders from PulseAudio sinks."""
        if self._grid is not None:
            self._grid.destroy()
        if self._sink_scales is not None:
            del self._sink_scales
        if self._sink_input_scales is not None:
            del self._sink_input_scales
        self._sink_scales = {}
        self._sink_input_scales = {}

        self._grid = Gtk.Grid()
        self._grid.set_column_spacing(2)
        self._grid.set_row_spacing(self.SPACING)
        self._frame.add(self._grid)

        pos = 0

        # touching pa objects here!
        pa_threaded_mainloop_lock(self._volctl.pa_mgr.mainloop)

        # sinks
        for _, sink in self._volctl.pa_mgr.pa_sinks.items():
            scale, icon = self._add_scale(sink)
            self._sink_scales[sink.idx] = scale
            scale.connect("value-changed", self._cb_sink_scale_change)
            self._update_scale_vol(scale, sink.volume, sink.mute)
            scale.set_margin_top(self.SPACING)
            icon.set_margin_bottom(self.SPACING)
            self._grid.attach(scale, pos, 0, 1, 1)
            self._grid.attach(icon, pos, 1, 1, 1)
            pos += 1

        # separator
        if self._volctl.pa_mgr.pa_sink_inputs:
            separator = Gtk.Separator().new(Gtk.Orientation.VERTICAL)
            separator.set_margin_top(self.SPACING)
            separator.set_margin_bottom(self.SPACING)
            self._grid.attach(separator, pos, 0, 1, 2)
            pos += 1

        # sink inputs
        for _, sink_input in self._volctl.pa_mgr.pa_sink_inputs.items():
            scale, icon = self._add_scale(sink_input)
            self._sink_input_scales[sink_input.idx] = scale
            scale.connect("value-changed", self._cb_sink_input_scale_change)
            self._update_scale_vol(scale, sink_input.volume, sink_input.mute)
            scale.set_margin_top(self.SPACING)
            icon.set_margin_bottom(self.SPACING)
            self._grid.attach(scale, pos, 0, 1, 1)
            self._grid.attach(icon, pos, 1, 1, 1)
            pos += 1

        pa_threaded_mainloop_unlock(self._volctl.pa_mgr.mainloop)

        self.show_all()
        self.resize(1, 1)  # smallest possible
        GObject.idle_add(self._set_position)

    def _add_scale(self, sink):
        # scale
        scale = Gtk.Scale().new(Gtk.Orientation.VERTICAL)
        scale.set_range(PA_VOLUME_MUTED, PA_VOLUME_NORM)
        scale.set_inverted(True)
        scale.set_size_request(24, 128)
        scale.set_tooltip_text(sink.name)
        self._set_increments_on_scale(scale)
        if self._show_percentage:
            scale.set_draw_value(True)
            scale.set_value_pos(Gtk.PositionType.BOTTOM)
            scale.connect("format_value", self._cb_format_value)
        else:
            scale.set_draw_value(False)

        if self._volctl.settings.get_boolean("vu-enabled"):
            scale.set_has_origin(False)
            scale.set_show_fill_level(False)
            scale.set_fill_level(0)
            scale.set_restrict_to_fill_level(False)

        # icon
        icon = Gtk.Image()
        icon.set_tooltip_text(sink.name)
        icon.set_from_icon_name(sink.icon_name, Gtk.IconSize.SMALL_TOOLBAR)

        return scale, icon

    @staticmethod
    def _update_scale_vol(scale, volume, mute):
        scale.set_value(volume)
        if mute is not None:
            scale.set_sensitive(not mute)

    @staticmethod
    def _update_scale_peak(scale, val):
        if val > 0:
            scale.set_show_fill_level(True)
            scale.set_fill_level(val * PA_VOLUME_NORM)
        else:
            scale.set_show_fill_level(False)
            scale.set_fill_level(0)

    def _enable_timeout(self):
        if self._volctl.settings.get_boolean("auto-close") and self._timeout is None:
            self._timeout = GLib.timeout_add(
                self._volctl.settings.get_int("timeout"), self._cb_auto_close
            )

    def _remove_timeout(self):
        if self._timeout is not None:
            GLib.Source.remove(self._timeout)
            self._timeout = None

    # called by pa thread

    def update_sink_scale(self, idx, volume, mute):
        """Update sink scale by index."""
        try:
            scale = self._sink_scales[idx]
        except KeyError:
            return
        self._update_scale_vol(scale, volume, mute)

    def update_sink_input_scale(self, idx, volume, mute):
        """Update sink input scale by index."""
        try:
            scale = self._sink_input_scales[idx]
        except KeyError:
            return
        self._update_scale_vol(scale, volume, mute)

    def update_sink_scale_peak(self, idx, val):
        """Update sink scale peak value by index."""
        try:
            scale = self._sink_scales[idx]
        except KeyError:
            return
        self._update_scale_peak(scale, val)

    def update_sink_input_scale_peak(self, idx, val):
        """Update sink input peak value by index."""
        try:
            scale = self._sink_input_scales[idx]
        except KeyError:
            return
        self._update_scale_peak(scale, val)

    # gui callbacks

    @staticmethod
    def _cb_format_value(scale, val):
        """Format scale label"""
        return "{:d}%".format(round(100 * val / PA_VOLUME_NORM))

    def _cb_sink_scale_change(self, scale):
        value = int(scale.get_value())
        idx = self._find_sink_idx_by_scale(scale)

        if idx >= 0:
            pa_threaded_mainloop_lock(self._volctl.pa_mgr.mainloop)
            sink = self._volctl.pa_mgr.pa_sinks[idx]
            sink.set_volume(value)
            pa_threaded_mainloop_unlock(self._volctl.pa_mgr.mainloop)

    def _cb_sink_input_scale_change(self, scale):
        value = int(scale.get_value())
        idx = self._find_sink_input_idx_by_scale(scale)

        if idx >= 0:
            pa_threaded_mainloop_lock(self._volctl.pa_mgr.mainloop)
            sink_input = self._volctl.pa_mgr.pa_sink_inputs[idx]
            sink_input.set_volume(value)
            pa_threaded_mainloop_unlock(self._volctl.pa_mgr.mainloop)

    def _cb_enter_notify(self, win, event):
        if (
            event.detail == Gdk.NotifyType.NONLINEAR
            or event.detail == Gdk.NotifyType.NONLINEAR_VIRTUAL
        ):
            self._remove_timeout()

    def _cb_leave_notify(self, win, event):
        if (
            event.detail == Gdk.NotifyType.NONLINEAR
            or event.detail == Gdk.NotifyType.NONLINEAR_VIRTUAL
        ):
            self._enable_timeout()

    def _cb_auto_close(self):
        self._timeout = None
        self._volctl.close_slider()
        return GLib.SOURCE_REMOVE

    # find sinks

    def _find_sink_idx_by_scale(self, scale):
        return self._find_idx_by_scale(scale, self._sink_scales)

    def _find_sink_input_idx_by_scale(self, scale):
        return self._find_idx_by_scale(scale, self._sink_input_scales)

    @staticmethod
    def _find_idx_by_scale(scale, scales):
        for idx, val in scales.items():
            if scale == val:
                return idx
        return -1
