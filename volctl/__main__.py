"""volctl main entry point."""
from .tray import VolCtlTray


def main():
    """volctl main entry point."""
    import gi
    gi.require_version('Gtk', '3.0')
    from gi.repository import Gtk
    VolCtlTray()
    Gtk.main()


main()
