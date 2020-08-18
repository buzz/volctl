# pylint: disable=import-outside-toplevel

"""volctl main entry point."""


"""Start volctl."""
import gi

gi.require_version("Gdk", "3.0")
gi.require_version("GdkX11", "3.0")
gi.require_version("Gio", "2.0")
gi.require_version("GLib", "2.0")
gi.require_version("GObject", "2.0")
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk
from volctl.app import VolctlApp


def main():
    Gtk.init()
    VolctlApp()
    Gtk.main()
