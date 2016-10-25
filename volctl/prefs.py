import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, Gdk, Gio


class PreferencesDialog(Gtk.Dialog):

    def __init__(self, settings):
        Gtk.Dialog.__init__(self, 'Preferences')
        self.settings = settings
        self.settings_schema = Gio.SettingsSchemaSource.get_default().lookup(
            'apps.volctl', False)
        self.settings.connect('changed', self.cb_settings_changed)

        self.set_type_hint(Gdk.WindowTypeHint.NORMAL)
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
        label.set_markup('<b>volctl settings</b>')
        hbox.pack_start(label, False, True, 10)
        listbox.add(row)

        # auto-hide volume sliders
        k = self.settings_schema.get_key('auto-close')
        row = Gtk.ListBoxRow()
        row.set_tooltip_text(k.get_description())

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label(k.get_summary(), xalign=0)
        vbox.pack_start(label, True, True, 0)
        switch = Gtk.Switch()
        switch.props.valign = Gtk.Align.CENTER
        self.settings.bind(
            'auto-close', switch, 'active', Gio.SettingsBindFlags.DEFAULT)
        hbox.pack_start(switch, False, True, 10)

        listbox.add(row)

        # auto-hide timeout
        k = self.settings_schema.get_key('timeout')
        row = Gtk.ListBoxRow()
        row.set_tooltip_text(k.get_description())

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label('  ' + k.get_summary(), xalign=0)
        vbox.pack_start(label, True, True, 0)
        scale = Gtk.Scale().new(Gtk.Orientation.HORIZONTAL)
        r = k.get_range()
        scale.set_range(r[1][0], r[1][1])
        scale.set_digits(False)
        scale.set_size_request(128, 24)
        scale.connect('format_value', self.scale_timeout_format)
        self.settings.bind('timeout', scale.get_adjustment(), 'value',
                           Gio.SettingsBindFlags.DEFAULT)
        hbox.pack_start(scale, False, True, 10)
        self.row_timeout = row

        listbox.add(row)

        # toggle solo/mute buttons
        k = self.settings_schema.get_key('solo-mute')
        row = Gtk.ListBoxRow()
        row.set_tooltip_text(k.get_description())

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label(k.get_summary(), xalign=0)
        vbox.pack_start(label, True, True, 0)
        switch = Gtk.Switch()
        switch.props.valign = Gtk.Align.CENTER
        self.settings.bind(
            'solo-mute', switch, 'active', Gio.SettingsBindFlags.DEFAULT)
        hbox.pack_start(switch, False, True, 10)

        listbox.add(row)

        # mouse wheel step
        k = self.settings_schema.get_key('mouse-wheel-step')
        row = Gtk.ListBoxRow()
        row.set_tooltip_text(k.get_description())

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label(k.get_summary(), xalign=0)
        vbox.pack_start(label, True, True, 0)
        scale = Gtk.Scale().new(Gtk.Orientation.HORIZONTAL)
        r = k.get_range()
        scale.set_range(r[1][0], r[1][1])
        scale.set_digits(False)
        scale.set_size_request(128, 24)
        scale.connect('format_value', self.scale_mouse_wheel_step_format)
        self.settings.bind('mouse-wheel-step', scale.get_adjustment(), 'value',
                           Gio.SettingsBindFlags.DEFAULT)
        hbox.pack_start(scale, False, True, 10)

        listbox.add(row)

        # mixer command
        k = self.settings_schema.get_key('mixer-command')
        row = Gtk.ListBoxRow()
        row.set_tooltip_text(k.get_description())

        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        row.add(hbox)
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        hbox.pack_start(vbox, True, True, 10)

        label = Gtk.Label(k.get_summary(), xalign=0)
        vbox.pack_start(label, True, True, 0)
        entry = Gtk.Entry().new()
        self.settings.bind(
            'mixer-command', entry, 'text', Gio.SettingsBindFlags.DEFAULT)
        hbox.pack_start(entry, False, True, 10)

        listbox.add(row)

        self.show_all()
        self.set_timeout_show()

    def scale_timeout_format(self, scale, value):
        return '%.1f sec' % (value / 1000.0)

    def scale_mouse_wheel_step_format(self, scale, value):
        return '%.1f %%' % (100.0 / value)

    def set_timeout_show(self):
        if self.settings.get_boolean('auto-close'):
            self.row_timeout.show()
        else:
            self.row_timeout.hide()

    # gsettings callback

    def cb_settings_changed(self, settings, key):
        if key == 'auto-close':
            self.set_timeout_show()
