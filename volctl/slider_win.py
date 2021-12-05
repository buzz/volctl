"""
VolumeSliders window

Small window that appears next to tray icon when activated. It show sliders
for main and application volume.
"""

from gi.repository import Gtk, Gdk, GLib
from pulsectl.pulsectl import c


class VolumeSliders(Gtk.Window):
    """Window that displays volume sliders."""

    SPACING = 6

    # Time without receiving an update after which a peak value should be reset to 0
    PEAK_TIMEOUT = 100  # ms

    def __init__(self, volctl, xpos, ypos):
        super().__init__(type=Gtk.WindowType.POPUP)
        self._volctl = volctl
        self._xpos, self._ypos = xpos, ypos
        self._grid = None
        self._show_percentage = self._volctl.settings.get_boolean("show-percentage")

        # GUI objects by index
        self._sink_scales = {}
        self._sink_input_scales = {}

        self.connect("enter-notify-event", self._cb_enter_notify)
        self.connect("leave-notify-event", self._cb_leave_notify)
        self.connect("destroy", self._cb_destroy)

        self._frame = Gtk.Frame()
        self._frame.set_shadow_type(Gtk.ShadowType.OUT)
        self.add(self._frame)
        self.create_widgets()

        # Timeout
        self._timeout = None
        self._enable_timeout()

        # Peak monitoring timeouts
        self._peak_timeouts = {}

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
        try:
            step = self._volctl.mouse_wheel_step / 100.0
            scale.set_increments(step, step)
        except AttributeError:
            # Pop-up might have closed already.
            pass

    def _set_position(self):
        if self._xpos == 0 and self._ypos == 0:
            # Bogus event positions, happens on Gnome and maybe others,
            # use current mouse pointer position instead.
            _, self._xpos, self._ypos = (
                Gdk.Display.get_default()
                .get_default_seat()
                .get_pointer()
                .get_position()
            )

        monitor = Gdk.Display.get_default().get_monitor_at_point(self._xpos, self._ypos)
        monitor_rect = monitor.get_workarea()

        status_icon = self._volctl.status_icon
        info_avail, screen, status_rect, orient = status_icon.get_geometry()
        if not info_avail:
            # If status icon geometry is not available we need to assume values
            status_rect = Gdk.Rectangle()
            status_rect.x, status_rect.y = self._xpos, self._ypos
            status_rect.width = status_rect.height = 1
        win_w, win_h = self.get_size()

        # Initial position (window anchor based on screen quadrant)
        win_x = status_rect.x
        win_y = status_rect.y
        if status_rect.x - monitor_rect.x < monitor_rect.width / 2:
            win_x += status_rect.width
        else:
            if orient == Gtk.Orientation.VERTICAL:
                win_x -= win_w
        if status_rect.y - monitor_rect.y < monitor_rect.height / 2:
            win_y += status_rect.height
        else:
            win_y -= win_h

        # Keep window inside screen
        if win_x + win_w > monitor_rect.x + monitor_rect.width:
            win_x = monitor_rect.x + monitor_rect.width - win_w

        if screen:
            self.set_screen(screen)
        self.move(win_x, win_y)

    def create_widgets(self):
        """Create base widgets."""
        self._grid = Gtk.Grid()
        self._grid.set_column_spacing(2)
        self._grid.set_row_spacing(self.SPACING)
        self._frame.add(self._grid)
        self.recreate_sliders()

    def clear_sliders(self):
        """Remove all children from grid layout."""
        self._sink_scales = {}
        self._sink_input_scales = {}
        while True:
            if self._grid.get_child_at(0, 0) is None:
                break
            self._grid.remove_column(0)

    def recreate_sliders(self):
        """Recreate sliders from PulseAudio sinks."""
        self.clear_sliders()
        pos = 0

        with self._volctl.pulsemgr.pulse() as pulse:
            try:
                sinks = pulse.sink_list()
                sink_inputs = pulse.sink_input_list()
            except c.pa.CallError:
                print("Warning: Could not get sinks/sink inputs")
                sinks = []
                sink_inputs = []

        # Sinks
        for sink in sinks:
            for prop_name in ["alsa.card_name", "device.description"]:
                try:
                    card_name = sink.proplist[prop_name]
                    break
                except KeyError:
                    continue
                card_name = sink.name

            props = (
                card_name,
                "audio-card",
                sink.volume.value_flat,
                sink.mute,
            )
            scale, btn = self._add_scale(pos, props)
            self._sink_scales[sink.index] = scale, btn
            idx = sink.index
            scale.connect("value-changed", self._cb_sink_scale_change, idx)
            btn.connect("toggled", self._cb_sink_mute_toggle, idx)
            pos += 1

        # Sink inputs
        if sink_inputs:
            separator = Gtk.Separator().new(Gtk.Orientation.VERTICAL)
            separator.set_margin_top(self.SPACING)
            separator.set_margin_bottom(self.SPACING)
            self._grid.attach(separator, pos, 0, 1, 2)
            pos += 1

            for sink_input in sink_inputs:
                name, icon_name = self._name_icon_name_from_sink_input(sink_input)
                props = name, icon_name, sink_input.volume.value_flat, sink_input.mute
                scale, btn = self._add_scale(pos, props)
                self._sink_input_scales[sink_input.index] = scale, btn
                idx = sink_input.index
                scale.connect("value-changed", self._cb_sink_input_scale_change, idx)
                btn.connect("toggled", self._cb_sink_input_mute_toggle, idx)
                pos += 1

        self.show_all()
        self.resize(1, 1)  # Smallest possible
        GLib.idle_add(self._set_position)

    def _add_scale(self, pos, props):
        name, icon_name, val, mute = props
        # Scale
        scale = Gtk.Scale().new(Gtk.Orientation.VERTICAL)
        scale.set_range(0.0, 1.0)
        scale.set_inverted(True)
        scale.set_size_request(24, 128)
        scale.set_margin_top(self.SPACING)
        scale.set_tooltip_markup(name)
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

        # Mute button
        icon = Gtk.Image()
        icon.set_from_icon_name(icon_name, Gtk.IconSize.SMALL_TOOLBAR)
        btn = Gtk.ToggleButton()
        btn.set_image(icon)
        btn.set_relief(Gtk.ReliefStyle.NONE)
        btn.set_margin_bottom(self.SPACING)
        btn.set_tooltip_markup(name)

        self._update_scale_values((scale, btn), val, mute)
        self._grid.attach(scale, pos, 0, 1, 1)
        self._grid.attach(btn, pos, 1, 1, 1)
        return scale, btn

    @staticmethod
    def _name_icon_name_from_sink_input(sink_input):
        proplist = sink_input.proplist
        try:
            name = f"<b>{proplist['application.name']}</b>: {proplist['media.name']}"
        except KeyError:
            try:
                name = proplist["application.name"]
            except KeyError:
                name = sink_input.name
        try:
            icon_name = proplist["media.icon_name"]
        except KeyError:
            try:
                icon_name = proplist["application.icon_name"]
            except KeyError:
                icon_name = "multimedia-volume-control"
        return name, icon_name

    @staticmethod
    def _update_scale_values(scale_btn, volume, mute):
        scale, btn = scale_btn
        scale.set_value(volume)
        if mute is not None:
            scale.set_sensitive(not mute)
            btn.set_active(mute)

    @staticmethod
    def _update_scale_peak(scale, val):
        if val > 0:
            scale.set_show_fill_level(True)
            scale.set_fill_level(val)
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

    # Updates coming from outside

    def update_sink_scale(self, idx, volume, mute):
        """Update sink scale by index."""
        try:
            scale_btn = self._sink_scales[idx]
        except KeyError:
            return
        self._update_scale_values(scale_btn, volume, mute)

    def update_sink_input_scale(self, idx, volume, mute):
        """Update sink input scale by index."""
        try:
            scale_btn = self._sink_input_scales[idx]
        except KeyError:
            return
        self._update_scale_values(scale_btn, volume, mute)

    def update_scale_peak(self, idx, val):
        """Update scale peak value by index on a sink or sink input scale."""
        try:
            scale, _ = self._sink_scales[idx]
            val = val * scale.get_value()  # Need to scale into range for sinks
        except KeyError:
            try:
                scale, _ = self._sink_input_scales[idx]
            except KeyError:
                return
        self._update_scale_peak(scale, val)

        # If a sound source is paused, peak updates stop coming in. To prevent
        # to show a stale peak vale, we set peak=0 after a timeout.
        try:
            GLib.Source.remove(self._peak_timeouts[idx])
        except KeyError:
            pass
        timeout = GLib.timeout_add(self.PEAK_TIMEOUT, self._cb_peak_reset, idx)
        self._peak_timeouts[idx] = timeout

    def _scale_change(self, scale, idx, sink_input=False):
        value = scale.get_value()
        with self._volctl.pulsemgr.pulse() as pulse:
            list_method = pulse.sink_input_list if sink_input else pulse.sink_list
            try:
                sink = next(s for s in list_method() if s.index == idx)
                if sink:
                    pulse.volume_set_all_chans(sink, value)
            except c.pa.CallError as err:
                print(f"Warning: Could not set volume on {idx}: {err}")

    # GUI callbacks

    def _cb_destroy(self, win):
        self._remove_timeout()
        for timeout in self._peak_timeouts.values():
            GLib.Source.remove(timeout)

    @staticmethod
    def _cb_format_value(scale, val):
        """Format scale label"""
        return str(round(100 * val))

    def _cb_sink_scale_change(self, scale, idx):
        self._scale_change(scale, idx)

    def _cb_sink_input_scale_change(self, scale, idx):
        self._scale_change(scale, idx, sink_input=True)

    def _cb_sink_mute_toggle(self, button, idx):
        mute = button.get_property("active")
        self._volctl.pulsemgr.sink_set_mute(idx, mute)

    def _cb_sink_input_mute_toggle(self, button, idx):
        mute = button.get_property("active")
        self._volctl.pulsemgr.sink_input_set_mute(idx, mute)

    def _cb_enter_notify(self, win, event):
        if event.detail in (Gdk.NotifyType.NONLINEAR, Gdk.NotifyType.NONLINEAR_VIRTUAL):
            self._remove_timeout()

    def _cb_leave_notify(self, win, event):
        if event.detail in (Gdk.NotifyType.NONLINEAR, Gdk.NotifyType.NONLINEAR_VIRTUAL):
            self._enable_timeout()

    def _cb_auto_close(self):
        self._timeout = None
        self._volctl.close_slider()
        return GLib.SOURCE_REMOVE

    def _cb_peak_reset(self, idx):
        del self._peak_timeouts[idx]
        scale = None
        try:
            scale, _ = self._sink_scales[idx]
        except KeyError:
            try:
                scale, _ = self._sink_input_scales[idx]
            except KeyError:
                pass
        if scale:
            self._update_scale_peak(scale, 0.0)
        return GLib.SOURCE_REMOVE
