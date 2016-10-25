import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, Gdk


class PreferencesDialog(Gtk.Dialog):

    def __init__(self):
        Gtk.Dialog.__init__(self, "Preferences")
        box = self.get_content_area()
        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        box.pack_start(hbox, True, True, 20)

        listbox = Gtk.ListBox()
        listbox.set_selection_mode(Gtk.SelectionMode.NONE)
        hbox.pack_start(listbox, True, True, 10)
        row = Gtk.ListBoxRow()
        row.set_activatable(False)
        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        label = Gtk.Label(xalign=0)
        label.set_markup("<b>volctl settings</b>")
        hbox.pack_start(label, False, True, 10)
        listbox.add(row)

        # auto-hide volume sliders
        row = Gtk.ListBoxRow()

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label("Automatically close volume control", xalign=0)
        vbox.pack_start(label, True, True, 0)
        switch = Gtk.Switch()
        switch.props.valign = Gtk.Align.CENTER
        hbox.pack_start(switch, False, True, 10)

        listbox.add(row)

        # toggle solo/mute buttons
        row = Gtk.ListBoxRow()

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label("Show mute/solo buttons", xalign=0)
        vbox.pack_start(label, True, True, 0)
        switch = Gtk.Switch()
        switch.props.valign = Gtk.Align.CENTER
        hbox.pack_start(switch, False, True, 10)

        listbox.add(row)

        # mouse wheel step
        row = Gtk.ListBoxRow()

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label("Mouse wheel step", xalign=0)
        vbox.pack_start(label, True, True, 0)
        scale = Gtk.Scale().new(Gtk.Orientation.HORIZONTAL)
        scale.set_range(5, 25)
        scale.set_digits(False)
        scale.set_size_request(128, 24)
        hbox.pack_start(scale, False, True, 10)

        listbox.add(row)

        # mixer command
        row = Gtk.ListBoxRow()

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label("Custom mixer command (leave empty for pavucontrol)", xalign=0)
        vbox.pack_start(label, True, True, 0)
        entry = Gtk.Entry().new()
        hbox.pack_start(entry, False, True, 10)

        listbox.add(row)

        self.show_all()
