"""volctl preference dialog"""
from gi.repository import Gtk, Gdk, Gio


class PreferencesDialog(Gtk.Dialog):
    """Preferences dialog for volctl"""

    def __init__(self, settings, default_mixer_cmd):
        super().__init__("Preferences")
        self._settings = settings
        self._schema = settings.get_property("settings-schema")
        self._default_mixer_cmd = default_mixer_cmd
        self._row_timeout = None
        self._row_osd_timeout = None
        self._settings.connect("changed", self._cb_settings_changed)
        self._setup_ui()

    def _setup_ui(self):
        self.set_type_hint(Gdk.WindowTypeHint.NORMAL)

        box = self.get_content_area()
        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        box.pack_start(hbox, True, True, 20)

        self.listbox = Gtk.ListBox()
        self.listbox.set_selection_mode(Gtk.SelectionMode.NONE)
        hbox.pack_start(self.listbox, True, True, 10)
        row = Gtk.ListBoxRow()
        row.set_activatable(False)
        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        label = Gtk.Label(xalign=0)
        label.set_markup("<b>volctl settings</b>")
        hbox.pack_start(label, False, True, 10)
        self.listbox.add(row)

        self._add_switch("show-percentage")
        self._add_switch("auto-close")
        self._row_timeout = self._add_scale("timeout", self._scale_timeout_format)
        self._add_scale("mouse-wheel-step", self._scale_mouse_wheel_step_format)
        self._add_switch("osd-enabled")
        self._row_osd_timeout = self._add_scale(
            "osd-timeout", self._scale_timeout_format
        )
        self._row_osd_size = self._add_scale("osd-scale", self._scale_osd_size_format)
        self._add_switch("vu-enabled")
        self._add_entry("mixer-command", self._default_mixer_cmd)

        self._update_rows()
        self.set_position(Gtk.WindowPosition.CENTER_ALWAYS)
        self.show_all()

    def _add_switch(self, name):
        key = self._schema.get_key(name)
        row = Gtk.ListBoxRow()
        row.set_tooltip_text(key.get_description())

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label(key.get_summary(), xalign=0)
        vbox.pack_start(label, True, True, 0)
        switch = Gtk.Switch()
        switch.props.valign = Gtk.Align.CENTER
        self._settings.bind(name, switch, "active", Gio.SettingsBindFlags.DEFAULT)
        hbox.pack_start(switch, False, True, 10)

        self.listbox.add(row)

    def _add_scale(self, name, format_func):
        key = self._schema.get_key(name)
        row = Gtk.ListBoxRow()
        row.set_tooltip_text(key.get_description())

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label("  " + key.get_summary(), xalign=0)
        vbox.pack_start(label, True, True, 0)
        scale = Gtk.Scale().new(Gtk.Orientation.HORIZONTAL)
        key_range = key.get_range()
        scale.set_range(key_range[1][0], key_range[1][1])
        scale.set_digits(False)
        scale.set_size_request(128, 24)
        scale.connect("format_value", format_func)
        self._settings.bind(
            name,
            scale.get_adjustment(),
            "value",
            Gio.SettingsBindFlags.DEFAULT,
        )
        hbox.pack_start(scale, False, True, 10)
        self.listbox.add(row)
        return row

    def _add_entry(self, name, placeholder):
        key = self._schema.get_key(name)
        row = Gtk.ListBoxRow()
        row.set_tooltip_text(key.get_description())

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label(key.get_summary(), xalign=0)
        vbox.pack_start(label, True, True, 0)
        entry = Gtk.Entry().new()
        entry.set_placeholder_text(placeholder)
        self._settings.bind(name, entry, "text", Gio.SettingsBindFlags.DEFAULT)
        hbox.pack_start(entry, False, True, 10)

        self.listbox.add(row)

    @staticmethod
    def _scale_timeout_format(_, value):
        return "%.1f sec" % (value / 1000.0)

    @staticmethod
    def _scale_osd_size_format(_, value):
        return "%d %%" % (value,)

    @staticmethod
    def _scale_mouse_wheel_step_format(_, value):
        return "%.1f %%" % (100.0 / value)

    def _update_rows(self):
        if self._settings.get_boolean("auto-close"):
            self._row_timeout.show()
        else:
            self._row_timeout.hide()
        if self._settings.get_boolean("osd-enabled"):
            self._row_osd_timeout.show()
            self._row_osd_size.show()
        else:
            self._row_osd_timeout.hide()
            self._row_osd_size.hide()

    # gsettings callback

    def _cb_settings_changed(self, settings, key):
        self._update_rows()
