"""volctl main entry point."""


def main():
    """volctl main entry point."""
    import gi
    gi.require_version('Gtk', '3.0')
    from gi.repository import Gtk
    from volctl.app import VolctlApp
    VolctlApp()
    Gtk.main()


if __name__ == '__main__':
    main()
