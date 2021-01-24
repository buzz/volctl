from math import floor
from gi.repository import Gtk, Gdk, GLib


class StatusIcon(Gtk.StatusIcon):
    """Volume control status icon."""

    def __init__(self, volctl):
        super().__init__()
        self._volctl = volctl
        self._menu = None
        GLib.idle_add(self._setup_statusicon)

    def update(self, volume, mute):
        """Update status icon according to volume state."""
        if mute:
            state = "muted"
        else:
            idx = min(int(floor(volume * 3)), 2)
            state = ["low", "medium", "high"][idx]
        icon_name = f"audio-volume-{state}"
        self.set_from_icon_name(icon_name)

    # GUI setup

    def _setup_statusicon(self):
        self._setup_menu()
        self.set_visible(True)
        self.set_name("volctl")
        self.set_title("Volume")
        self.set_has_tooltip(True)
        self.connect("popup-menu", self._cb_popup)
        self.connect("button-press-event", self._cb_button_press)
        self.connect("scroll-event", self._cb_scroll)
        self.connect("query-tooltip", self._cb_tooltip)
        self.connect("notify::embedded", self._cb_notify_embedded)

    def _setup_menu(self):
        self._menu = Gtk.Menu()
        mute_menu_item = Gtk.ImageMenuItem("Mute")
        img = Gtk.Image.new_from_icon_name(
            "audio-volume-muted", Gtk.IconSize.SMALL_TOOLBAR
        )
        mute_menu_item.set_image(img)
        mute_menu_item.connect("activate", self._cb_menu_mute)

        mixer_menu_item = Gtk.ImageMenuItem("Mixer")
        img = Gtk.Image.new_from_icon_name(
            "multimedia-volume-control", Gtk.IconSize.SMALL_TOOLBAR
        )
        mixer_menu_item.set_image(img)
        mixer_menu_item.connect("activate", self._cb_menu_mixer)

        preferences_menu_item = Gtk.ImageMenuItem("Preferences")
        img = Gtk.Image.new_from_icon_name(
            "preferences-desktop", Gtk.IconSize.SMALL_TOOLBAR
        )
        preferences_menu_item.set_image(img)
        preferences_menu_item.connect("activate", self._cb_menu_preferences)

        about_menu_item = Gtk.ImageMenuItem.new_from_stock(Gtk.STOCK_ABOUT)
        about_menu_item.connect("activate", self._cb_menu_about)

        exit_menu_item = Gtk.ImageMenuItem.new_from_stock(Gtk.STOCK_QUIT)
        exit_menu_item.connect("activate", self._cb_menu_quit)

        self._menu.append(mute_menu_item)
        self._menu.append(mixer_menu_item)
        self._menu.append(preferences_menu_item)
        self._menu.append(Gtk.SeparatorMenuItem())
        self._menu.append(about_menu_item)
        self._menu.append(exit_menu_item)
        self._menu.show_all()

    # GUI callbacks

    def _cb_notify_embedded(self, status_icon, embedded):
        if embedded:
            try:
                vol, mute = self._volctl.pulsemgr.volume, self._volctl.pulsemgr.mute
            except AttributeError:
                return
            self.update(vol, mute)

    def _cb_tooltip(self, item, xcoord, ycoord, keyboard_mode, tooltip):
        # pylint: disable=too-many-arguments
        perc = self._volctl.pulsemgr.volume * 100
        text = f"Volume: {perc:.0f}%"
        if self._volctl.pulsemgr.mute:
            text += ' <span weight="bold">(muted)</span>'
        tooltip.set_markup(text)
        return True

    def _cb_menu_mute(self, widget):
        self._volctl.pulsemgr.toggle_main_mute()

    def _cb_menu_mixer(self, widget):
        self._volctl.launch_mixer()

    def _cb_menu_preferences(self, widget):
        self._volctl.show_preferences()

    def _cb_menu_about(self, widget):
        self._volctl.show_about()

    def _cb_menu_quit(self, widget):
        self._volctl.quit()

    def _cb_scroll(self, widget, event):
        old_vol = self._volctl.pulsemgr.volume
        amount = 1.0 / self._volctl.mouse_wheel_step
        if event.direction == Gdk.ScrollDirection.DOWN:
            amount *= -1
        elif event.direction == Gdk.ScrollDirection.UP:
            pass
        else:
            return
        new_value = old_vol + amount
        new_value = min(1.0, new_value)
        new_value = max(0.0, new_value)

        # User action prolongs auto-close timer
        if self._volctl.sliders_win is not None:
            self._volctl.sliders_win.reset_timeout()

        self._volctl.pulsemgr.set_main_volume(new_value)

    def _cb_button_press(self, widget, event):
        if event.button == 1:
            if event.type == Gdk.EventType.BUTTON_PRESS:
                if not self._volctl.close_slider():
                    monitor = Gdk.Display.get_default().get_monitor_at_point(
                        event.x_root, event.y_root
                    )
                    monitor_rect = monitor.get_workarea()
                    self._volctl.show_slider(monitor_rect)
            if event.type == Gdk.EventType.DOUBLE_BUTTON_PRESS:
                self._volctl.launch_mixer()

    def _cb_popup(self, icon, button, time):
        self._menu.popup(None, None, None, None, button, time)
