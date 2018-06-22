import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, Gdk, GLib

from .lib_pulseaudio import (PA_VOLUME_MUTED, PA_VOLUME_NORM,
    pa_threaded_mainloop_lock, pa_threaded_mainloop_unlock)


class VolumeSlider:
    def __init__(self, volctl):
        self.volctl = volctl
        self.win = Gtk.Window(type=Gtk.WindowType.POPUP)
        self.win.connect('enter-notify-event', self.cb_enter_notify)
        self.win.connect('leave-notify-event', self.cb_leave_notify)
        self.grid = Gtk.Grid()
        self.grid.set_column_spacing(2)
        self.grid.set_row_spacing(6)
        self.frame = Gtk.Frame()
        self.frame.set_shadow_type(Gtk.ShadowType.OUT)
        self.frame.add(self.grid)
        self.win.add(self.frame)

        # gui objects by index
        self.sink_scales = {}
        self.sink_input_scales = {}

        self.create_sliders()
        self.win.show_all()
        self.set_position()

        # timeout
        self.auto_close_timeout = None
        self.enable_timeout()

    def _find_idx_by_scale(self, scale, scales):
        for idx, v in scales.iteritems():
            if scale == v:
                return idx
        # should never happen
        raise Exception('Sink index not found for scale!')

    def _find_sink_idx_by_scale(self, scale):
        return self._find_idx_by_scale(scale, self.sink_scales)

    def _find_sink_input_idx_by_scale(self, scale):
        return self._find_idx_by_scale(scale, self.sink_input_scales)

    def set_position(self):
        a, screen, rect, orient = self.volctl.statusicon.get_geometry()
        win_width, win_height = self.win.get_size()
        monitor = screen.get_monitor_geometry(
            screen.get_monitor_at_window(screen.get_active_window()))
        
        # slider window should not leave screen boundaries
        x = rect.x
        if x + win_width > monitor.width:
            x = monitor.width - win_width
            self.win.move(x, rect.y)
        # top or bottom panel?
        if rect.y > monitor.height / 2:
            self.win.move(x, rect.y - win_height)
        else:
            self.win.move(x, rect.y + rect.height)

    def create_sliders(self):
        x = 0

        # touching pa objects here!
        pa_threaded_mainloop_lock(self.volctl.pa_mgr.pa.pa_mainloop)

        # sinks
        for idx, sink in self.volctl.pa_mgr.pa_sinks.iteritems():
            scale, icon = self.add_scale(sink)
            self.sink_scales[sink.idx] = scale
            scale.connect('value-changed', self.cb_sink_scale)
            self.update_scale(scale, sink.volume, sink.mute)
            scale.set_margin_top(6)
            icon.set_margin_bottom(6)
            self.grid.attach(scale, x, 0, 1, 1)
            self.grid.attach(icon, x, 1, 1, 1)
            x += 1

        # separator
        if len(self.volctl.pa_mgr.pa_sink_inputs) > 0:
            separator = Gtk.Separator().new(Gtk.Orientation.VERTICAL)
            separator.set_margin_top(6)
            separator.set_margin_bottom(6)
            self.grid.attach(separator, x, 0, 1, 2)
            x += 1

        # sink inputs
        for idx, sink_input in self.volctl.pa_mgr.pa_sink_inputs.iteritems():
            scale, icon = self.add_scale(sink_input)
            self.sink_input_scales[sink_input.idx] = scale
            scale.connect('value-changed', self.cb_sink_input_scale)
            self.update_scale(scale, sink_input.volume, sink_input.mute)
            scale.set_margin_top(6)
            icon.set_margin_bottom(6)
            self.grid.attach(scale, x, 0, 1, 1)
            self.grid.attach(icon, x, 1, 1, 1)
            x += 1

        pa_threaded_mainloop_unlock(self.volctl.pa_mgr.pa.pa_mainloop)

    def add_scale(self, sink):
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

    def set_increments(self):
        for idx, scale in self.sink_scales.iteritems():
            self._set_increments_on_scale(scale)
        for idx, scale in self.sink_input_scales.iteritems():
            self._set_increments_on_scale(scale)

    def _set_increments_on_scale(self, scale):
        scale.set_increments(PA_VOLUME_NORM / self.volctl.mouse_wheel_step,
                             PA_VOLUME_NORM / self.volctl.mouse_wheel_step)

    def update_scale(self, scale, volume, mute):
        scale.set_value(volume)
        if not mute is None:
            scale.set_sensitive(not mute)

    def enable_timeout(self):
        if self.volctl.settings.get_boolean('auto-close') and \
           self.auto_close_timeout is None:
            self.auto_close_timeout = GLib.timeout_add(
                self.volctl.settings.get_int('timeout'), self._auto_close)

    def remove_timeout(self):
        if not self.auto_close_timeout is None:
            GLib.Source.remove(self.auto_close_timeout)
            self.auto_close_timeout = None

    def reset_timeout(self):
        self.remove_timeout()
        self.enable_timeout()

    # called by pa thread

    def update_sink_scale(self, idx, volume, mute):
        try:
            scale = self.sink_scales[idx]
        except KeyError:
            return
        self.update_scale(scale, volume, mute)

    def update_sink_input_scale(self, idx, volume, mute):
        try:
            scale = self.sink_input_scales[idx]
        except KeyError:
            return
        self.update_scale(scale, volume, mute)

    # gui callbacks

    def cb_sink_scale(self, scale):
        value = int(scale.get_value())
        idx = self._find_sink_idx_by_scale(scale)

        m = self.volctl.pa_mgr.pa.pa_mainloop
        pa_threaded_mainloop_lock(m)
        sink = self.volctl.pa_mgr.pa_sinks[idx]
        sink.set_volume(value)
        pa_threaded_mainloop_unlock(m)

    def cb_sink_input_scale(self, scale):
        value = int(scale.get_value())
        idx = self._find_sink_input_idx_by_scale(scale)

        m = self.volctl.pa_mgr.pa.pa_mainloop
        pa_threaded_mainloop_lock(m)
        sink_input = self.volctl.pa_mgr.pa_sink_inputs[idx]
        sink_input.set_volume(value)
        pa_threaded_mainloop_unlock(m)

    def cb_enter_notify(self, win, event):
        if event.detail == Gdk.NotifyType.NONLINEAR or \
           event.detail == Gdk.NotifyType.NONLINEAR_VIRTUAL:
            self.remove_timeout()

    def cb_leave_notify(self, win, event):
        if event.detail == Gdk.NotifyType.NONLINEAR or \
           event.detail == Gdk.NotifyType.NONLINEAR_VIRTUAL:
            self.enable_timeout()

    def close(self):
        self.win.destroy()

    def _auto_close(self):
        self.auto_close_timeout = None
        self.close()
        return GLib.SOURCE_REMOVE
