"""volctl tray icon"""

from math import floor
from gi.repository import Gtk, Gdk

from volctl.lib.pulseaudio import (
    PA_VOLUME_MUTED,
    PA_VOLUME_NORM,
    pa_threaded_mainloop_lock,
    pa_threaded_mainloop_unlock,
)


class TrayIcon(Gtk.StatusIcon):
    """Volume control tray icon."""

    def __init__(self, volctl):
        super().__init__()
        self.initialized = False
        self._volctl = volctl
        self._volume = 0
        self._mute = False
        self._menu = Gtk.Menu()
        self._setup_statusicon()
        self._setup_menu()

    def update_values(self, volume, mute):
        """Remember current volume and mute values."""
        self._volume = volume
        self._mute = mute
        self._update_icon()
        # Consider completely initialized when first volume update was processed
        self.initialized = True

    def _update_icon(self):
        """Update status icon according to volume state."""
        value = float(self._volume) / float(PA_VOLUME_NORM)
        if self._mute:
            state = "muted"
        else:
            idx = min(int(floor(value * 3)), 2)
            state = ["low", "medium", "high"][idx]
        icon_name = "audio-volume-%s" % state
        self.set_from_icon_name(icon_name)

    # gui setup

    def _setup_statusicon(self):
        self.set_title("Volume")
        self.set_has_tooltip(True)
        self.connect("popup-menu", self._cb_popup)
        self.connect("button-press-event", self._cb_button_press)
        self.connect("scroll-event", self._cb_scroll)
        self.connect("query-tooltip", self._cb_tooltip)

    def _setup_menu(self):
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

    # gui callbacks

    def _cb_tooltip(self, item, xcoord, ycoord, keyboard_mode, tooltip):
        # pylint: disable=too-many-arguments
        perc = float(self._volume) / float(PA_VOLUME_NORM) * 100
        text = "Volume: %.0f%%" % perc
        if self._mute:
            text += ' <span weight="bold">(muted)</span>'
        tooltip.set_markup(text)
        return True

    def _cb_menu_mute(self, widget):
        pa_threaded_mainloop_lock(self._volctl.pa_mgr.mainloop)
        self._volctl.pa_mgr.toggle_main_mute()
        pa_threaded_mainloop_unlock(self._volctl.pa_mgr.mainloop)

    def _cb_menu_mixer(self, widget):
        self._volctl.launch_mixer()

    def _cb_menu_preferences(self, widget):
        self._volctl.show_preferences()

    def _cb_menu_about(self, widget):
        self._volctl.show_about()

    def _cb_menu_quit(self, widget):
        self._volctl.quit()

    def _cb_scroll(self, widget, event):
        old_vol = self._volume
        amount = PA_VOLUME_NORM / self._volctl.mouse_wheel_step
        if event.direction == Gdk.ScrollDirection.DOWN:
            amount *= -1
        elif event.direction == Gdk.ScrollDirection.UP:
            pass
        else:
            return
        new_value = old_vol + amount
        new_value = min(PA_VOLUME_NORM, new_value)
        new_value = max(PA_VOLUME_MUTED, new_value)
        new_value = int(new_value)

        # user action prolongs auto-close timer
        if self._volctl.sliders_win is not None:
            self._volctl.sliders_win.reset_timeout()

        pa_threaded_mainloop_lock(self._volctl.pa_mgr.mainloop)
        self._volctl.pa_mgr.set_main_volume(new_value)
        pa_threaded_mainloop_unlock(self._volctl.pa_mgr.mainloop)

    def _cb_button_press(self, widget, event):
        if event.button == 1:
            if event.type == Gdk.EventType.BUTTON_PRESS:
                if not self._volctl.close_slider():
                    self._volctl.show_slider()
            if event.type == Gdk.EventType.DOUBLE_BUTTON_PRESS:
                self._volctl.launch_mixer()

    def _cb_popup(self, icon, button, time):
        self._menu.popup(None, None, None, None, button, time)
