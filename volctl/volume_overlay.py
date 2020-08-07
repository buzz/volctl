"""
OSD volume overlay

A transparent OSD volume indicator for the bottom-right corner.

Various code snippets taken from https://github.com/kozec/sc-controller
"""

import cairo
from gi.repository import Gdk, Gtk, GdkX11

import volctl.lib.xwrappers as X


class VolumeOverlay(Gtk.Window):
    """Window that displays volume sliders."""

    WIDTH = 200
    HEIGHT = 200
    MARGIN = 20

    def __init__(self, volctl):
        super(VolumeOverlay, self).__init__()
        self.volctl = volctl
        self.position = (-self.MARGIN, -self.MARGIN)
        self.set_default_size(self.WIDTH, self.HEIGHT)

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
        self.connect("draw", self.draw_osd)

        self.show()

    def draw_osd(self, _, cr):
        """Draw on-screen volume display."""
        # transparent background
        cr.set_source_rgba(0.2, 0.2, 0.2, 0.8)
        cr.set_operator(cairo.OPERATOR_SOURCE)
        cr.paint()
        cr.set_operator(cairo.OPERATOR_OVER)

        # text
        text = "20 %"
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.9)
        cr.select_font_face("sans-serif")
        cr.set_font_size(42)
        _, _, width, _, _, _ = cr.text_extents(text)
        cr.move_to(self.WIDTH / 2 - width / 2, self.HEIGHT - 12)
        cr.show_text(text)

    def compute_position(self):
        """Adjusts position for currently active screen (display)."""
        xpos, ypos = self.position
        width, height = self.get_window_size()
        geometry = self.get_active_screen_geometry()
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

    def make_window_clicktrough(self):
        """Make events pass through window."""
        dpy = X.Display(hash(GdkX11.x11_get_default_xdisplay()))
        win = X.XID(self.get_window().get_xid())
        reg = X.create_region(dpy, None, 0)
        X.set_window_shape_region(dpy, win, X.SHAPE_BOUNDING, 0, 0, 0)
        X.set_window_shape_region(dpy, win, X.SHAPE_INPUT, 0, 0, reg)
        X.destroy_region(dpy, reg)

    def get_active_screen_geometry(self):
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

    def get_window_size(self):
        return self.get_window().get_width(), self.get_window().get_height()

    def show(self):
        """Show window."""
        self.realize()
        self.get_window().set_override_redirect(True)

        xpos, ypos = self.compute_position()
        if xpos < 0:  # Negative X position is counted from right border
            xpos = Gdk.Screen.width() - self.get_allocated_width() + xpos + 1
        if ypos < 0:  # Negative Y position is counted from bottom border
            ypos = Gdk.Screen.height() - self.get_allocated_height() + ypos + 1

        self.move(xpos, ypos)
        Gtk.Window.show(self)
        self.make_window_clicktrough()
