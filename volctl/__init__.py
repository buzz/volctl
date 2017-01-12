import sys
import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, GObject

from .tray import VolCtlTray


def main():
    GObject.threads_init()
    vctray = VolCtlTray()
    Gtk.main()
