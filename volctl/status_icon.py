from math import floor
import gi
from gi.repository import Gtk, Gdk, GLib

try:
    gi.require_version("StatusNotifier", "1.0")
    from gi.repository import StatusNotifier
except (ImportError, ValueError):
    StatusNotifier = None

from volctl.meta import PROGRAM_NAME


class StatusIcon:
    """
    Volume control status icon.

    By default StatusNotifier library is used if available. It uses DBUS and
    doesn't rely on XEmbed being available (Wayland support). As a fallback
    Gtk.StatusIcon will be used.

    Both have slightly different appearance and usability.
    """

    MAX_EMBED_ATTEMPTS = 5

    def __init__(self, volctl, prefer_gtksi):
        super().__init__()
        self._volctl = volctl
        self._menu = None

        self._current_impl = None

        # Prefer statusnotifier as it works under Gnome/KDE and also Wayland
        # unless overridden by user
        self._available_impl = []
        if StatusNotifier is not None:
            self._available_impl.append("sni")
        if prefer_gtksi:
            self._available_impl.append("gtksi")
        else:
            self._available_impl.insert(0, "gtksi")

        self._check_embed_timeout = None
        self._embed_attempts = 0
        self._instance = None

        self._create_menu()
        self._create_statusicon()

    def update(self, volume, mute):
        """Update status icon according to volume state."""
        icon_name = self._get_icon_name(volume, mute)
        if self._instance:
            if self._current_impl == "sni":
                self._set_sni_tooltip()
                self._instance.set_from_icon_name(
                    StatusNotifier.Icon.STATUS_NOTIFIER_ICON, icon_name
                )
            elif self._current_impl == "gtksi":
                self._instance.set_from_icon_name(icon_name)

    def get_geometry(self):
        """Return status icon position and size."""
        if self._instance and self._current_impl == "gtksi":
            return self._instance.get_geometry()
        # In case of statusnotifier, we don't have access to the icons geometry
        return False, None, None, None

    # GUI setup

    def _create_statusicon(self):
        """Attempt to create a status icon using the preferred
        implementation."""
        self._embed_attempts = 0
        try:
            self._current_impl = self._available_impl.pop()
        except IndexError:
            print(
                "Fatal error: Could not create a status icon. "
                "Are you sure you have a working notification area?"
            )
            self._volctl.quit()

        getattr(self, f"_create_{self._current_impl}")()

    def _create_sni(self):
        self._instance = StatusNotifier.Item.new_from_icon_name(
            "volctl",
            StatusNotifier.Category.HARDWARE,
            self._get_icon_name(
                self._volctl.pulsemgr.volume, self._volctl.pulsemgr.mute
            ),
        )
        self._instance.set_title(PROGRAM_NAME)
        self._instance.set_status(StatusNotifier.Status.ACTIVE)
        self._instance.set_item_is_menu(False)
        self._instance.set_context_menu(self._menu)
        self._instance.connect("activate", self._cb_sni_on_activate)
        self._instance.connect("secondary-activate", self._cb_sni_on_secondary_activate)
        self._instance.connect(
            "registration-failed", self._cb_sni_on_registration_failed
        )
        self._instance.connect("scroll", self._cb_sni_on_scroll)
        self._set_sni_tooltip()
        self._instance.register()

    def _create_gtksi(self):
        self._instance = Gtk.StatusIcon()
        self._instance.set_visible(True)
        self._instance.set_name("volctl")
        self._instance.set_title("Volume")
        self._instance.set_has_tooltip(True)
        self._instance.connect("popup-menu", self._cb_gtksi_popup)
        self._instance.connect("button-press-event", self._cb_gtksi_button_press)
        self._instance.connect("scroll-event", self._cb_gtksi_scroll)
        self._instance.connect("query-tooltip", self._cb_gtksi_tooltip)
        self._instance.connect("notify::embedded", self._cb_gtksi_notify_embedded)
        self._check_embed_timeout = GLib.timeout_add(100, self._cb_gtski_check_timeout)

    def _create_menu(self):
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

    @staticmethod
    def _get_icon_name(volume, mute):
        if mute:
            state = "muted"
        else:
            idx = min(int(floor(volume * 3)), 2)
            state = ["low", "medium", "high"][idx]
        return f"audio-volume-{state}"

    def _set_sni_tooltip(self):
        self._instance.freeze_tooltip()
        self._instance.set_tooltip_title("Volume")
        self._instance.set_tooltip_body(self._get_tooltip_markup())
        self._instance.thaw_tooltip()

    def _get_tooltip_markup(self):
        """Create tooltip markup."""
        perc = self._volctl.pulsemgr.volume * 100
        text = f"Volume: {perc:.0f}%"
        if self._volctl.pulsemgr.mute:
            text += ' <span weight="bold">(muted)</span>'
        return text

    # Callback actions

    def _cb_activate(self, xpos, ypos):
        if not self._volctl.close_slider():
            self._volctl.show_slider(xpos, ypos)

    def _cb_scroll(self, direction):
        amount = direction * 1.0 / self._volctl.mouse_wheel_step
        new_value = self._volctl.pulsemgr.volume + amount
        new_value = min(1.0, new_value)
        new_value = max(0.0, new_value)

        # User action prolongs auto-close timer
        if self._volctl.sliders_win is not None:
            self._volctl.sliders_win.reset_timeout()

        self._volctl.pulsemgr.set_main_volume(new_value)

    # Menu callbacks

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

    # statusnotifier callbacks

    def _cb_sni_on_activate(self, statusnotifer, posx, posy):
        self._cb_activate(posx, posy)

    def _cb_sni_on_secondary_activate(self, statusnotifer, posx, posy):
        self._volctl.pulsemgr.toggle_main_mute()

    def _cb_sni_on_scroll(self, statusnotifier, delta, orient):
        if orient == StatusNotifier.ScrollOrientation.VERTICAL:
            self._cb_scroll(-delta)

    def _cb_sni_on_registration_failed(self, statusnotifier, error):
        state = self._instance.get_state()
        if state == StatusNotifier.State.FAILED:
            print("Warning: Could not register StatusNotifierItem.")
            GLib.idle_add(self._create_statusicon)
        elif state == StatusNotifier.State.REGISTERING:
            self._check_embed_timeout = GLib.timeout_add(
                100, self._cb_sni_check_timeout
            )

    def _cb_sni_check_timeout(self):
        state = self._instance.get_state()
        if state == StatusNotifier.State.REGISTERED:
            return GLib.SOURCE_REMOVE
        if (
            self._embed_attempts > self.MAX_EMBED_ATTEMPTS
            or state == StatusNotifier.State.FAILED
        ):
            print("Warning: Could not register StatusNotifierItem.")
            self._instance = None
            GLib.idle_add(self._create_statusicon)
            return GLib.SOURCE_REMOVE
        self._embed_attempts += 1
        return GLib.SOURCE_CONTINUE

    # GTK.StatusIcon callbacks

    def _cb_gtksi_notify_embedded(self, status_icon, embedded):
        if embedded:
            if self._check_embed_timeout:
                GLib.Source.remove(self._check_embed_timeout)
            try:
                vol, mute = self._volctl.pulsemgr.volume, self._volctl.pulsemgr.mute
            except AttributeError:
                return
            self.update(vol, mute)

    def _cb_gtksi_tooltip(self, item, xcoord, ycoord, keyboard_mode, tooltip):
        # pylint: disable=too-many-arguments
        tooltip.set_markup(self._get_tooltip_markup())
        return True

    def _cb_gtksi_scroll(self, widget, event):
        if event.direction == Gdk.ScrollDirection.DOWN:
            self._cb_scroll(-1)
        elif event.direction == Gdk.ScrollDirection.UP:
            self._cb_scroll(1)

    def _cb_gtksi_button_press(self, widget, event):
        if event.button == 1:
            if event.type == Gdk.EventType.BUTTON_PRESS:
                self._cb_activate(event.x_root, event.y_root)
            if event.type == Gdk.EventType.DOUBLE_BUTTON_PRESS:
                self._volctl.launch_mixer()

    def _cb_gtksi_popup(self, icon, button, time):
        self._menu.popup(None, None, None, None, button, time)

    def _cb_gtski_check_timeout(self):
        if self._embed_attempts > self.MAX_EMBED_ATTEMPTS:
            print("Warning: Could not embed Gtk.StatusIcon.")
            self._instance = None
            GLib.idle_add(self._create_statusicon)
            return GLib.SOURCE_REMOVE
        if self._instance.is_embedded():
            return GLib.SOURCE_REMOVE
        self._embed_attempts += 1
        return GLib.SOURCE_CONTINUE
