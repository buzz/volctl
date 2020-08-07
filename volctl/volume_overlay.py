"""
OSD volume overlay

A transparent OSD volume indicator for the bottom-right corner.

Various code snippets taken from https://github.com/kozec/sc-controller
"""

import math
import cairo
from gi.repository import Gdk, Gtk, GdkX11, GLib

from volctl.lib.pulseaudio import PA_VOLUME_NORM
import volctl.lib.xwrappers as X


class VolumeOverlay(Gtk.Window):
    """Window that displays volume sliders."""

    WIDTH = 200
    HEIGHT = 200
    SCREEN_MARGIN = 64
    PADDING = 24
    BG_OPACITY = 0.85
    BG_CORNER_RADIUS = 8
    MUTE_OPACITY = 0.2
    TEXT_OPACITY = 0.8
    NUM_BARS = 16

    def __init__(self, volctl):
        super(VolumeOverlay, self).__init__()
        self.volctl = volctl
        self.position = (-self.SCREEN_MARGIN, -self.SCREEN_MARGIN)
        self.set_default_size(self.WIDTH, self.HEIGHT)
        self._volume = 0
        self._mute = False
        self._hide_timeout = None
        self._fadeout_timeout = None
        self._opacity = 0.0

        self.set_decorated(False)
        self.stick()
        self.set_skip_taskbar_hint(True)
        self.set_skip_pager_hint(True)
        self.set_keep_above(True)
        self.set_type_hint(Gdk.WindowTypeHint.NOTIFICATION)
        self.set_resizable(False)

        self.screen = self.get_screen()
        self.visual = self.screen.get_rgba_visual()
        if self.visual is not None and self.screen.is_composited():
            self.set_visual(self.visual)

        self.set_app_paintable(True)
        self.connect("draw", self._draw_osd)

        self.show()

    def update_values(self, volume, mute):
        """Remember current volume and mute values."""
        self._volume = volume
        self._mute = mute
        self._unhide()
        if self._hide_timeout is not None:
            GLib.Source.remove(self._hide_timeout)
        self._hide_timeout = GLib.timeout_add(
            self.volctl.settings.get_int("timeout"), self._cb_hide_timeout
        )

    def show(self):
        """Show window."""
        self.realize()
        self.get_window().set_override_redirect(True)

        xpos, ypos = self._compute_position()
        if xpos < 0:  # Negative X position is counted from right border
            xpos = Gdk.Screen.width() - self.get_allocated_width() + xpos + 1
        if ypos < 0:  # Negative Y position is counted from bottom border
            ypos = Gdk.Screen.height() - self.get_allocated_height() + ypos + 1

        self.move(xpos, ypos)
        Gtk.Window.show(self)
        self._make_window_clicktrough()

    def _draw_osd(self, _, cr):
        """Draw on-screen volume display."""
        val = float(self._volume) / float(PA_VOLUME_NORM)
        mute_opacity = self.MUTE_OPACITY if self._mute else 1.0
        xcenter = self.WIDTH / 2

        # background
        deg = math.pi / 180.0
        cr.new_sub_path()
        cr.arc(
            self.WIDTH - self.BG_CORNER_RADIUS,
            self.BG_CORNER_RADIUS,
            self.BG_CORNER_RADIUS,
            -90 * deg,
            0,
        )
        cr.arc(
            self.WIDTH - self.BG_CORNER_RADIUS,
            self.HEIGHT - self.BG_CORNER_RADIUS,
            self.BG_CORNER_RADIUS,
            0,
            90 * deg,
        )
        cr.arc(
            self.BG_CORNER_RADIUS,
            self.HEIGHT - self.BG_CORNER_RADIUS,
            self.BG_CORNER_RADIUS,
            90 * deg,
            180 * deg,
        )
        cr.arc(
            self.BG_CORNER_RADIUS,
            self.BG_CORNER_RADIUS,
            self.BG_CORNER_RADIUS,
            180 * deg,
            270 * deg,
        )
        cr.close_path()

        cr.set_source_rgba(0.1, 0.1, 0.1, self.BG_OPACITY * self._opacity)
        cr.set_operator(cairo.OPERATOR_SOURCE)
        cr.fill()
        cr.set_operator(cairo.OPERATOR_OVER)

        # color
        cr.set_source_rgba(
            1.0, 1.0, 1.0, self.TEXT_OPACITY * mute_opacity * self._opacity
        )

        # text
        text = "{:d} %".format(round(100 * val))
        cr.select_font_face("sans-serif")
        cr.set_font_size(42)
        _, _, text_width, text_height, _, _ = cr.text_extents(text)
        cr.move_to(xcenter - text_width / 2, self.HEIGHT - self.PADDING)
        cr.show_text(text)

        # volume indicator
        ind_height = self.HEIGHT - 3 * self.PADDING - text_height
        outer_radius = ind_height / 2
        inner_radius = outer_radius / 1.618
        bars = min(round(self.NUM_BARS * val), self.NUM_BARS)
        cr.set_line_width(5)
        cr.set_line_cap(cairo.LINE_CAP_ROUND)
        for i in range(bars):
            cr.identity_matrix()
            cr.translate(xcenter, self.PADDING + ind_height / 2)
            cr.rotate(math.pi + 2 * math.pi / self.NUM_BARS * i)
            cr.move_to(0.0, -inner_radius)
            cr.line_to(0.0, -outer_radius)
            cr.stroke()

    def _compute_position(self):
        """Adjusts position for currently active screen (display)."""
        xpos, ypos = self.position
        width, height = self._get_window_size()
        geometry = self._get_active_screen_geometry()
        if geometry:
            if xpos < 0:
                xpos = xpos + geometry.x + geometry.width - width
            else:
                xpos = xpos + geometry.x
            if ypos < 0:
                ypos = ypos + geometry.y + geometry.height - height
            else:
                ypos = geometry.y + ypos

        return xpos, ypos

    def _make_window_clicktrough(self):
        """Make events pass through window."""
        dpy = X.Display(hash(GdkX11.x11_get_default_xdisplay()))
        win = X.XID(self.get_window().get_xid())
        reg = X.create_region(dpy, None, 0)
        X.set_window_shape_region(dpy, win, X.SHAPE_BOUNDING, 0, 0, 0)
        X.set_window_shape_region(dpy, win, X.SHAPE_INPUT, 0, 0, reg)
        X.destroy_region(dpy, reg)

    def _get_active_screen_geometry(self):
        """
        Returns geometry of active screen or None if active screen
        cannot be determined.
        """
        screen = self.get_window().get_screen()
        active_window = screen.get_active_window()
        if active_window:
            monitor = screen.get_monitor_at_window(active_window)
            if monitor is not None:
                return screen.get_monitor_geometry(monitor)
        return None

    def _get_window_size(self):
        return self.get_window().get_width(), self.get_window().get_height()

    def _hide(self):
        self._fadeout_timeout = GLib.timeout_add(30, self._cb_fadeout_timeout)
        self.queue_draw()

    def _unhide(self):
        if self._fadeout_timeout is not None:
            GLib.Source.remove(self._fadeout_timeout)
            self._fadeout_timeout = None
        self._opacity = 1.0
        self.queue_draw()

    def _cb_fadeout_timeout(self):
        self._opacity -= 0.05
        self.queue_draw()
        if self._opacity >= 0:
            return True
        self._opacity = 0.0
        self._fadeout_timeout = None
        return False

    def _cb_hide_timeout(self):
        self._hide_timeout = None
        self._hide()
