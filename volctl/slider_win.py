"""
VolumeSliders window


Small window that appears next to tray icon when activated. It displays
master and app volume sliders.
"""

from gi.repository import Gtk, Gdk, GLib

from volctl.lib.pulseaudio import (
    PA_VOLUME_MUTED,
    PA_VOLUME_NORM,
    pa_threaded_mainloop_lock,
    pa_threaded_mainloop_unlock,
)


class VolumeSliders(Gtk.Window):
    """Window that displays volume sliders."""

    SPACING = 6

    def __init__(self, volctl):
        super().__init__(type=Gtk.WindowType.POPUP)
        self._volctl = volctl

        self.connect("enter-notify-event", self._cb_enter_notify)
        self.connect("leave-notify-event", self._cb_leave_notify)

        self._grid = Gtk.Grid()
        self._grid.set_column_spacing(2)
        self._grid.set_row_spacing(self.SPACING)
        self._frame = Gtk.Frame()
        self._frame.set_shadow_type(Gtk.ShadowType.OUT)
        self._frame.add(self._grid)
        self.add(self._frame)

        # gui objects by index
        self._sink_scales = {}
        self._sink_input_scales = {}

        self._create_sliders()
        self._set_position()
        self.show_all()

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
        _, screen, rect, _ = self._volctl.tray_icon.get_geometry()
        self.set_screen(screen)
        width, height = self.get_size()
        monitor = screen.get_monitor_geometry(
            screen.get_monitor_at_window(screen.get_active_window())
        )

        # slider window should not leave screen boundaries
        xcoord = rect.x
        if xcoord + width > monitor.width:
            xcoord = monitor.width - width
            self.move(xcoord, rect.y)
        # top or bottom panel?
        if rect.y > monitor.height / 2:
            self.move(xcoord, rect.y - height)
        else:
            self.move(xcoord, rect.y + rect.height)

    def _create_sliders(self):
        pos = 0

        # touching pa objects here!
        pa_threaded_mainloop_lock(self._volctl.pa_mgr.mainloop)

        # sinks
        for _, sink in self._volctl.pa_mgr.pa_sinks.items():
            scale, icon = self._add_scale(sink)
            self._sink_scales[sink.idx] = scale
            scale.connect("value-changed", self._cb_sink_scale_change)
            self._update_scale(scale, sink.volume, sink.mute)
            scale.set_margin_top(self.SPACING)
            icon.set_margin_bottom(self.SPACING)
            self._grid.attach(scale, pos, 0, 1, 1)
            self._grid.attach(icon, pos, 1, 1, 1)
            pos += 1

        # separator
        if not self._volctl.pa_mgr.pa_sink_inputs:
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
            self._update_scale(scale, sink_input.volume, sink_input.mute)
            scale.set_margin_top(self.SPACING)
            icon.set_margin_bottom(self.SPACING)
            self._grid.attach(scale, pos, 0, 1, 1)
            self._grid.attach(icon, pos, 1, 1, 1)
            pos += 1

        pa_threaded_mainloop_unlock(self._volctl.pa_mgr.mainloop)

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
        self._update_scale(scale, volume, mute)

    def update_sink_input_scale(self, idx, volume, mute):
        """Update sink input scale by index."""
        try:
            scale = self._sink_input_scales[idx]
        except KeyError:
            return
        self._update_scale(scale, volume, mute)

    # gui callbacks

    def _cb_sink_scale_change(self, scale):
        value = int(scale.get_value())
        idx = self._find_sink_idx_by_scale(scale)

        pa_threaded_mainloop_lock(self._volctl.pa_mgr.mainloop)
        sink = self._volctl.pa_mgr.pa_sinks[idx]
        sink.set_volume(value)
        pa_threaded_mainloop_unlock(self._volctl.pa_mgr.mainloop)

    def _cb_sink_input_scale_change(self, scale):
        value = int(scale.get_value())
        idx = self._find_sink_input_idx_by_scale(scale)

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
        # should never happen
        raise ValueError("Sink index not found for scale!")
