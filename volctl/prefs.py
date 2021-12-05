"""volctl preference dialog"""
from gi.repository import Gtk, Gdk, Gio


class PreferencesDialog(Gtk.Dialog):
    """Preferences dialog for volctl"""

    MARGIN = 8
    COL_SPACING = 24
    ROW_SPACING = 12

    def __init__(self, settings, default_mixer_cmd):
        super().__init__("Preferences")

        self._settings = settings
        self._schema = settings.get_property("settings-schema")
        self._default_mixer_cmd = default_mixer_cmd
        self._row_timeout = None
        self._row_osd_timeout = None
        self._settings.connect("changed", self._cb_settings_changed)

        self.add_button(Gtk.STOCK_CLOSE, Gtk.ResponseType.OK)
        self._grid_top = 0
        self._setup_ui()
        self.set_icon_name("preferences-desktop")
        self.set_resizable(False)
        self.set_position(Gtk.WindowPosition.CENTER)

    def _setup_ui(self):
        self.set_type_hint(Gdk.WindowTypeHint.NORMAL)

        box = self.get_content_area()
        box.set_margin_top(self.MARGIN)
        box.set_margin_bottom(self.MARGIN)
        box.set_margin_start(self.MARGIN)
        box.set_margin_end(self.MARGIN)

        self.grid = Gtk.Grid()
        self.grid.set_margin_bottom(self.MARGIN * 2)
        self.grid.set_column_spacing(self.COL_SPACING)
        self.grid.set_row_spacing(self.ROW_SPACING)
        self.grid.set_column_homogeneous(True)
        self.grid.set_row_homogeneous(False)

        # Tray icon options
        self._create_section_label("Tray icon")
        self._add_scale("mouse-wheel-step", self._scale_mouse_wheel_step_format)
        self._add_entry("mixer-command", placeholder=self._default_mixer_cmd)
        self._add_switch("prefer-gtksi")

        # Volume slider window options
        self._create_section_label("Volume sliders")
        self._add_switch("show-percentage")
        self._add_switch("vu-enabled")
        self._add_switch("auto-close")
        self._row_timeout = self._add_scale("timeout", self._scale_timeout_format)

        # OSD options
        self._create_section_label("On-screen display")
        self._add_switch("osd-enabled")
        self._row_osd_timeout = self._add_scale(
            "osd-timeout", self._scale_timeout_format
        )
        self._row_osd_size = self._add_scale("osd-scale", self._scale_osd_size_format)
        self._row_osd_position = self._add_entry("osd-position", "")

        self._update_rows()
        box.pack_start(self.grid, False, True, 0)
        self.show_all()

    def _create_section_label(self, caption):
        label = Gtk.Label()
        label.set_markup(f"<b>{caption}</b>")
        self._attach(label, width=2)

    def _add_label(self, label_text, tooltip_text):
        label = Gtk.Label(label_text, xalign=0)
        label.set_tooltip_text(tooltip_text)
        label.set_margin_start(self.MARGIN * 2)
        self._attach(label, next_row=False)

    def _add_switch(self, name):
        key = self._schema.get_key(name)
        tooltip_text = key.get_description()
        self._add_label(key.get_summary(), tooltip_text)

        switch = Gtk.Switch()
        switch.set_tooltip_text(tooltip_text)
        self._settings.bind(name, switch, "active", Gio.SettingsBindFlags.DEFAULT)
        self._attach(switch, left=1)
        return switch

    def _add_scale(self, name, format_func):
        key = self._schema.get_key(name)
        tooltip_text = key.get_description()
        self._add_label(key.get_summary(), tooltip_text)

        scale = Gtk.Scale().new(Gtk.Orientation.HORIZONTAL)
        scale.set_tooltip_text(tooltip_text)
        key_range = key.get_range()
        scale.set_range(key_range[1][0], key_range[1][1])
        scale.set_digits(False)
        scale.set_size_request(128, 24)
        scale.connect("format_value", format_func)
        self._settings.bind(
            name, scale.get_adjustment(), "value", Gio.SettingsBindFlags.DEFAULT
        )
        self._attach(scale, left=1)
        return scale

    def _add_entry(self, name, placeholder=None):
        key = self._schema.get_key(name)
        tooltip_text = key.get_description()
        self._add_label(key.get_summary(), tooltip_text)

        entry = Gtk.Entry()
        entry.set_tooltip_text(tooltip_text)
        if placeholder:
            entry.set_placeholder_text(placeholder)
        self._settings.bind(name, entry, "text", Gio.SettingsBindFlags.DEFAULT)
        self._attach(entry, left=1)
        return entry

    def _attach(self, widget, left=0, width=1, next_row=True):
        halign = Gtk.Align.FILL
        if left == 0:
            halign = Gtk.Align.START
        if isinstance(widget, Gtk.Switch):
            halign = Gtk.Align.END
        widget.set_halign(halign)
        widget.set_valign(Gtk.Align.FILL)
        widget.set_hexpand(True)
        widget.set_vexpand(False)
        self.grid.attach(widget, left, self._grid_top, width, 1)
        if next_row:
            self._grid_top += 1

    @staticmethod
    def _scale_timeout_format(_, value):
        return f"{value/1000:.1f} sec"

    @staticmethod
    def _scale_osd_size_format(_, value):
        return f"{round(value)} %"

    @staticmethod
    def _scale_mouse_wheel_step_format(_, value):
        return f"{value:.1f} %"

    def _update_rows(self):
        if self._settings.get_boolean("auto-close"):
            self._row_timeout.set_sensitive(True)
        else:
            self._row_timeout.set_sensitive(False)

        if self._settings.get_boolean("osd-enabled"):
            self._row_osd_timeout.set_sensitive(True)
            self._row_osd_size.set_sensitive(True)
            self._row_osd_position.set_sensitive(True)
        else:
            self._row_osd_timeout.set_sensitive(False)
            self._row_osd_size.set_sensitive(False)
            self._row_osd_position.set_sensitive(False)

    # GSettings callback

    def _cb_settings_changed(self, settings, key):
        self._update_rows()
