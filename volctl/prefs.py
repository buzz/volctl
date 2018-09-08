"""volctl preference dialog"""

import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, Gdk, Gio


class PreferencesDialog(Gtk.Dialog):
    """Preferences dialog for volctl"""

    def __init__(self, settings):
        Gtk.Dialog.__init__(self, 'Preferences')

        self.settings = settings
        self.settings_schema = Gio.SettingsSchemaSource.get_default().lookup(
            'apps.volctl', False)
        self.settings.connect('changed', self._cb_settings_changed)

        self.row_timeout = None
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
        label.set_markup('<b>volctl settings</b>')
        hbox.pack_start(label, False, True, 10)
        self.listbox.add(row)

        self._setup_auto_hide()
        self._setup_auto_hide_timeout()
        self._setup_mouse_wheel_step()
        self._setup_mixer_command()

        self.show_all()
        self._set_timeout_show()

    def _setup_auto_hide(self):
        key = self.settings_schema.get_key('auto-close')
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
        self.settings.bind(
            'auto-close', switch, 'active', Gio.SettingsBindFlags.DEFAULT)
        hbox.pack_start(switch, False, True, 10)

        self.listbox.add(row)

    def _setup_auto_hide_timeout(self):
        key = self.settings_schema.get_key('timeout')
        row = Gtk.ListBoxRow()
        row.set_tooltip_text(key.get_description())

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label('  ' + key.get_summary(), xalign=0)
        vbox.pack_start(label, True, True, 0)
        scale = Gtk.Scale().new(Gtk.Orientation.HORIZONTAL)
        key_range = key.get_range()
        scale.set_range(key_range[1][0], key_range[1][1])
        scale.set_digits(False)
        scale.set_size_request(128, 24)
        scale.connect('format_value', self._scale_timeout_format)
        self.settings.bind('timeout', scale.get_adjustment(), 'value',
                           Gio.SettingsBindFlags.DEFAULT)
        hbox.pack_start(scale, False, True, 10)
        self.row_timeout = row

        self.listbox.add(row)

    def _setup_mouse_wheel_step(self):
        key = self.settings_schema.get_key('mouse-wheel-step')
        row = Gtk.ListBoxRow()
        row.set_tooltip_text(key.get_description())

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label(key.get_summary(), xalign=0)
        vbox.pack_start(label, True, True, 0)
        scale = Gtk.Scale().new(Gtk.Orientation.HORIZONTAL)
        key_range = key.get_range()
        scale.set_range(key_range[1][0], key_range[1][1])
        scale.set_digits(False)
        scale.set_size_request(128, 24)
        scale.connect('format_value', self._scale_mouse_wheel_step_format)
        self.settings.bind('mouse-wheel-step', scale.get_adjustment(), 'value',
                           Gio.SettingsBindFlags.DEFAULT)
        hbox.pack_start(scale, False, True, 10)

        self.listbox.add(row)

    def _setup_mixer_command(self):
        key = self.settings_schema.get_key('mixer-command')
        row = Gtk.ListBoxRow()
        row.set_tooltip_text(key.get_description())

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label(key.get_summary(), xalign=0)
        vbox.pack_start(label, True, True, 0)
        entry = Gtk.Entry().new()
        self.settings.bind(
            'mixer-command', entry, 'text', Gio.SettingsBindFlags.DEFAULT)
        hbox.pack_start(entry, False, True, 10)

        self.listbox.add(row)

    @staticmethod
    def _scale_timeout_format(_, value):
        return '%.1f sec' % (value / 1000.0)

    @staticmethod
    def _scale_mouse_wheel_step_format(_, value):
        return '%.1f %%' % (100.0 / value)

    def _set_timeout_show(self):
        if self.settings.get_boolean('auto-close'):
            self.row_timeout.show()
        else:
            self.row_timeout.hide()

    # gsettings callback

    def _cb_settings_changed(self, settings, key):
        if key == 'auto-close':
            self._set_timeout_show()
