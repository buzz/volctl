"""volctl main entry point."""
import gi

gi.require_version("Gdk", "3.0")
gi.require_version("GdkX11", "3.0")
gi.require_version("Gio", "2.0")
gi.require_version("GLib", "2.0")
gi.require_version("GObject", "2.0")
gi.require_version("Gtk", "3.0")
# pylint: disable=wrong-import-position
from gi.repository import Gtk
from volctl.app import VolctlApp


def main():
    """Start volctl."""
    app = None
    Gtk.init()
    try:
        app = VolctlApp()
        Gtk.main()
    except KeyboardInterrupt:
        app.quit()
