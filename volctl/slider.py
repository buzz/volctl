"""
VolumeSliders window


Small window that appears next to tray icon when activated. It displays
master and app volume sliders.
"""

import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, Gdk, GLib

from .lib_pulseaudio import (
    PA_VOLUME_MUTED, PA_VOLUME_NORM, pa_threaded_mainloop_lock,
    pa_threaded_mainloop_unlock,
)


class VolumeSliders:
    """Window that displays volume sliders."""

    SPACING = 6

    def __init__(self, volctl):
        self.volctl = volctl
        self.win = Gtk.Window(type=Gtk.WindowType.POPUP)
        self.win.connect('enter-notify-event', self._cb_enter_notify)
        self.win.connect('leave-notify-event', self._cb_leave_notify)
        self.grid = Gtk.Grid()
        self.grid.set_column_spacing(2)
        self.grid.set_row_spacing(self.SPACING)
        self.frame = Gtk.Frame()
        self.frame.set_shadow_type(Gtk.ShadowType.OUT)
        self.frame.add(self.grid)
        self.win.add(self.frame)

        # gui objects by index
        self.sink_scales = {}
        self.sink_input_scales = {}

        self._create_sliders()
        self.win.show_all()
        self._set_position()

        # timeout
        self.auto_close_timeout = None
        self._enable_timeout()

    def set_increments(self):
        """Set sliders increment step."""
        for _, scale in self.sink_scales.items():
            self._set_increments_on_scale(scale)
        for _, scale in self.sink_input_scales.items():
            self._set_increments_on_scale(scale)

    def reset_timeout(self):
        """Reset auto-close timeout."""
        self._remove_timeout()
        self._enable_timeout()

    def close(self):
        """Close slider."""
        self.win.destroy()

    def _set_increments_on_scale(self, scale):
        scale.set_increments(PA_VOLUME_NORM / self.volctl.mouse_wheel_step,
                             PA_VOLUME_NORM / self.volctl.mouse_wheel_step)

    def _set_position(self):
        _, screen, rect, _ = self.volctl.statusicon_geometry
        win_width, win_height = self.win.get_size()
        monitor = screen.get_monitor_geometry(
            screen.get_monitor_at_window(screen.get_active_window()))

        # slider window should not leave screen boundaries
        xcoord = rect.x
        if xcoord + win_width > monitor.width:
            xcoord = monitor.width - win_width
            self.win.move(xcoord, rect.y)
        # top or bottom panel?
        if rect.y > monitor.height / 2:
            self.win.move(xcoord, rect.y - win_height)
        else:
            self.win.move(xcoord, rect.y + rect.height)

    def _create_sliders(self):
        num = 0

        # touching pa objects here!
        pa_threaded_mainloop_lock(self.volctl.pa_mgr.pulseaudio.pa_mainloop)

        # sinks
        for _, sink in self.volctl.pa_mgr.pa_sinks.items():
            scale, icon = self._add_scale(sink)
            self.sink_scales[sink.idx] = scale
            scale.connect('value-changed', self._cb_sink_scale_change)
            self._update_scale(scale, sink.volume, sink.mute)
            scale.set_margin_top(self.SPACING)
            icon.set_margin_bottom(self.SPACING)
            self.grid.attach(scale, num, 0, 1, 1)
            self.grid.attach(icon, num, 1, 1, 1)
            num += 1

        # separator
        if not self.volctl.pa_mgr.pa_sink_inputs:
            separator = Gtk.Separator().new(Gtk.Orientation.VERTICAL)
            separator.set_margin_top(self.SPACING)
            separator.set_margin_bottom(self.SPACING)
            self.grid.attach(separator, num, 0, 1, 2)
            num += 1

        # sink inputs
        for _, sink_input in self.volctl.pa_mgr.pa_sink_inputs.items():
            scale, icon = self._add_scale(sink_input)
            self.sink_input_scales[sink_input.idx] = scale
            scale.connect('value-changed', self._cb_sink_input_scale_change)
            self._update_scale(scale, sink_input.volume, sink_input.mute)
            scale.set_margin_top(self.SPACING)
            icon.set_margin_bottom(self.SPACING)
            self.grid.attach(scale, num, 0, 1, 1)
            self.grid.attach(icon, num, 1, 1, 1)
            num += 1

        pa_threaded_mainloop_unlock(self.volctl.pa_mgr.pulseaudio.pa_mainloop)

    def _add_scale(self, sink):
        # scale
        scale = Gtk.Scale().new(Gtk.Orientation.VERTICAL)
        scale.set_draw_value(False)
        scale.set_value_pos(Gtk.PositionType.BOTTOM)
        scale.set_range(PA_VOLUME_MUTED, PA_VOLUME_NORM)
        scale.set_inverted(True)
        scale.set_size_request(24, 128)
        scale.set_tooltip_text(sink.name)
        self._set_increments_on_scale(scale)

        # icon
        icon = Gtk.Image()
        icon.set_tooltip_text(sink.name)
        icon.set_from_icon_name(sink.icon_name, Gtk.IconSize.SMALL_TOOLBAR)

        return scale, icon

    @staticmethod
    def _update_scale(scale, volume, mute):
        scale.set_value(volume)
        if mute is not None:
            scale.set_sensitive(not mute)

    def _enable_timeout(self):
        if self.volctl.settings.get_boolean('auto-close') and \
           self.auto_close_timeout is None:
            self.auto_close_timeout = GLib.timeout_add(
                self.volctl.settings.get_int('timeout'), self._auto_close)

    def _remove_timeout(self):
        if self.auto_close_timeout is not None:
            GLib.Source.remove(self.auto_close_timeout)
            self.auto_close_timeout = None

    # called by pa thread

    def update_sink_scale(self, idx, volume, mute):
        """Update sink scale by index."""
        try:
            scale = self.sink_scales[idx]
        except KeyError:
            return
        self._update_scale(scale, volume, mute)

    def update_sink_input_scale(self, idx, volume, mute):
        """Update sink input scale by index."""
        try:
            scale = self.sink_input_scales[idx]
        except KeyError:
            return
        self._update_scale(scale, volume, mute)

    # gui callbacks

    def _cb_sink_scale_change(self, scale):
        value = int(scale.get_value())
        idx = self._find_sink_idx_by_scale(scale)

        mainloop = self.volctl.pa_mgr.pulseaudio.pa_mainloop
        pa_threaded_mainloop_lock(mainloop)
        sink = self.volctl.pa_mgr.pa_sinks[idx]
        sink.set_volume(value)
        pa_threaded_mainloop_unlock(mainloop)

    def _cb_sink_input_scale_change(self, scale):
        value = int(scale.get_value())
        idx = self._find_sink_input_idx_by_scale(scale)

        mainloop = self.volctl.pa_mgr.pulseaudio.pa_mainloop
        pa_threaded_mainloop_lock(mainloop)
        sink_input = self.volctl.pa_mgr.pa_sink_inputs[idx]
        sink_input.set_volume(value)
        pa_threaded_mainloop_unlock(mainloop)

    def _cb_enter_notify(self, win, event):
        if event.detail == Gdk.NotifyType.NONLINEAR or \
           event.detail == Gdk.NotifyType.NONLINEAR_VIRTUAL:
            self._remove_timeout()

    def _cb_leave_notify(self, win, event):
        if event.detail == Gdk.NotifyType.NONLINEAR or \
           event.detail == Gdk.NotifyType.NONLINEAR_VIRTUAL:
            self._enable_timeout()

    def _auto_close(self):
        self.auto_close_timeout = None
        self.close()
        return GLib.SOURCE_REMOVE

    # find sinks

    def _find_sink_idx_by_scale(self, scale):
        return self._find_idx_by_scale(scale, self.sink_scales)

    def _find_sink_input_idx_by_scale(self, scale):
        return self._find_idx_by_scale(scale, self.sink_input_scales)

    @staticmethod
    def _find_idx_by_scale(scale, scales):
        for idx, val in scales.items():
            if scale == val:
                return idx
        # should never happen
        raise ValueError('Sink index not found for scale!')
