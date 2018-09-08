"""volctl tray icon"""

from math import floor
from subprocess import Popen
import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, Gdk, Gio

from .lib_pulseaudio import (
    PA_VOLUME_MUTED, PA_VOLUME_NORM,
    pa_threaded_mainloop_lock, pa_threaded_mainloop_unlock
)
from .pulseaudio import PulseAudioManager
from ._version import __version__
from .prefs import PreferencesDialog
from .slider import VolumeSliders


DEFAULT_MIXER_CMD = '/usr/bin/pavucontrol'

PROGRAM_NAME = 'Volume Control'
COPYRIGHT = '(c) buzz'
LICENSE = Gtk.License.GPL_2_0
COMMENTS = 'Per-application volume control for GNU/Linux desktops'
WEBSITE = 'https://buzz.github.io/volctl/'


# TODO: put app logic in to app class
# TODO: Mirror all settings in app class, other classes should not use settings
#       directly.
class VolCtlTray():
    """Volume control tray icon."""
    # pylint: disable=too-many-instance-attributes

    def __init__(self):

        self._settings = Gio.Settings('apps.volctl', path='/apps/volctl/')
        self._settings.connect('changed', self._cb_settings_changed)
        self._mouse_wheel_step = self._settings.get_int('mouse-wheel-step')

        self._volume = 0
        self._mute = False

        # status icon
        self._statusicon = Gtk.StatusIcon()
        self._setup_statusicon()

        self._menu = Gtk.Menu()
        self._setup_menu()

        # windows
        self._sliders_win = None
        self._about_win = None
        self._preferences = None

        self._pa_mgr = PulseAudioManager(self)

    # updates coming from pulseaudio

    def slider_count_changed(self):
        """Amount of sliders changed."""
        try:
            self._close_slider()
            self._show_slider()
        except AttributeError:
            pass

    def update_values(self, volume, mute):
        """Main sink values are reflected in status icon."""
        self._volume = volume
        self._mute = mute
        self._update_icon()

    def update_sink_scale(self, idx, volume, mute):
        """Notify sink scale if update is coming from pulseaudio."""
        try:
            self._sliders_win.update_sink_scale(idx, volume, mute)
        except AttributeError:
            pass

    def update_sink_input_scale(self, idx, volume, mute):
        """Notify sink input scale if update is coming from pulseaudio."""
        try:
            self._sliders_win.update_sink_input_scale(idx, volume, mute)
        except AttributeError:
            pass

    # gui setup

    def _setup_statusicon(self):
        self._statusicon.set_title('Volume')
        self._statusicon.set_has_tooltip(True)
        self._statusicon.connect('popup-menu', self._cb_popup)
        self._statusicon.connect('button-press-event', self._cb_button_press)
        self._statusicon.connect('scroll-event', self._cb_scroll)
        self._statusicon.connect('query-tooltip', self._cb_tooltip)

    def _setup_menu(self):
        mute_menu_item = Gtk.ImageMenuItem('Mute')
        img = Gtk.Image.new_from_icon_name(
            'audio-volume-muted', Gtk.IconSize.SMALL_TOOLBAR)
        mute_menu_item.set_image(img)
        mute_menu_item.connect('activate', self._cb_menu_mute)

        mixer_menu_item = Gtk.ImageMenuItem('Mixer')
        img = Gtk.Image.new_from_icon_name(
            'multimedia-volume-control', Gtk.IconSize.SMALL_TOOLBAR)
        mixer_menu_item.set_image(img)
        mixer_menu_item.connect('activate', self._cb_menu_mixer)

        preferences_menu_item = Gtk.ImageMenuItem('Preferences')
        img = Gtk.Image.new_from_icon_name(
            'preferences-desktop', Gtk.IconSize.SMALL_TOOLBAR)
        preferences_menu_item.set_image(img)
        preferences_menu_item.connect('activate', self._cb_menu_preferences)

        about_menu_item = Gtk.ImageMenuItem.new_from_stock(Gtk.STOCK_ABOUT)
        about_menu_item.connect('activate', self._cb_menu_about)

        exit_menu_item = Gtk.ImageMenuItem.new_from_stock(Gtk.STOCK_QUIT)
        exit_menu_item.connect('activate', self._cb_menu_quit)

        self._menu.append(mute_menu_item)
        self._menu.append(mixer_menu_item)
        self._menu.append(preferences_menu_item)
        self._menu.append(Gtk.SeparatorMenuItem())
        self._menu.append(about_menu_item)
        self._menu.append(exit_menu_item)
        self._menu.show_all()

    # gui

    def _update_icon(self):
        value = float(self._volume) / float(PA_VOLUME_NORM)
        if self._mute:
            state = 'muted'
        else:
            idx = min(int(floor(value * 3)), 2)
            state = ['low', 'medium', 'high'][idx]
        icon_name = 'audio-volume-%s' % state
        self._statusicon.set_from_icon_name(icon_name)

    def _launch_mixer(self):
        mixer_cmd = self._settings.get_string('mixer-command')
        if mixer_cmd == '':
            mixer_cmd = DEFAULT_MIXER_CMD
        Popen(mixer_cmd)

    # gui callbacks

    def _cb_tooltip(self, item, xcoord, ycoord, keyboard_mode, tooltip):
        # pylint: disable=too-many-arguments
        perc = float(self._volume) / float(PA_VOLUME_NORM) * 100
        text = 'Volume: %.0f%%' % perc
        if self._mute:
            text += ' <span weight="bold">(muted)</span>'
        tooltip.set_markup(text)
        return True

    def _cb_menu_mute(self, widget):
        mainloop = self._pa_mgr.pulseaudio.pa_mainloop
        pa_threaded_mainloop_lock(mainloop)
        self._pa_mgr.toggle_main_mute()
        pa_threaded_mainloop_unlock(mainloop)

    def _cb_menu_mixer(self, widget):
        self._launch_mixer()

    def _cb_menu_preferences(self, widget):
        try:
            self._preferences.present()
        except AttributeError:
            self._preferences = PreferencesDialog(self._settings)
            self._preferences.run()
            self._preferences.destroy()
            del self._preferences

    def _cb_menu_about(self, widget):
        try:
            self._about_win.present()
        except AttributeError:
            self._about_win = Gtk.AboutDialog()
            self._about_win.set_program_name(PROGRAM_NAME)
            self._about_win.set_version(__version__)
            self._about_win.set_copyright(COPYRIGHT)
            self._about_win.set_license_type(LICENSE)
            self._about_win.set_comments(COMMENTS)
            self._about_win.set_website(WEBSITE)
            self._about_win.set_logo_icon_name('audio-volume-high')
            self._about_win.run()
            self._about_win.destroy()
            del self._about_win

    def _cb_menu_quit(self, widget):
        if self._pa_mgr:
            self._pa_mgr.close()

        if Gtk.main_level() > 0:
            try:
                self._preferences.response(0)
            except AttributeError:
                pass
            try:
                self._about_win.close()
            except AttributeError:
                pass
            Gtk.main_quit()
        else:
            exit(1)

    def _cb_scroll(self, widget, event):
        old_vol = self._volume
        amount = PA_VOLUME_NORM / self._settings.get_int('mouse-wheel-step')
        if event.direction == Gdk.ScrollDirection.DOWN:
            amount *= -1
        elif event.direction == Gdk.ScrollDirection.UP:
            pass
        else:
            return
        new_value = old_vol + amount
        new_value = min(PA_VOLUME_NORM, new_value)
        new_value = max(PA_VOLUME_MUTED, new_value)

        # user action prolongs auto-close timer
        try:
            self._sliders_win.reset_timeout()
        except AttributeError:
            pass

        mainloop = self._pa_mgr.pulseaudio.pa_mainloop
        pa_threaded_mainloop_lock(mainloop)
        self._pa_mgr.set_main_volume(new_value)
        pa_threaded_mainloop_unlock(mainloop)

    def _cb_button_press(self, widget, event):
        if event.button == 1:
            if event.type == Gdk.EventType.BUTTON_PRESS:
                try:
                    self._close_slider()
                except AttributeError:
                    self._show_slider()
            if event.type == Gdk.EventType.DOUBLE_BUTTON_PRESS:
                self._launch_mixer()

    def _show_slider(self):
        self._sliders_win = VolumeSliders(self)

    def _close_slider(self):
        self._sliders_win.close()
        del self._sliders_win

    def _cb_popup(self, icon, button, time):
        self._menu.popup(None, None, None, None, button, time)

    # gsettings callback

    def _cb_settings_changed(self, settings, key):
        if key == 'mouse-wheel-step':
            self._mouse_wheel_step = settings.get_int('mouse-wheel-step')
            try:
                self._sliders_win.set_increments()
            except AttributeError:
                pass

    # some properties are accessible from outside

    @property
    def mouse_wheel_step(self):
        """Get increment for one mouse wheel tick."""
        return self._mouse_wheel_step

    @property
    def statusicon_geometry(self):
        """Get status icon geometry."""
        return self._statusicon.get_geometry()

    @property
    def settings(self):
        """Get GSettings instance."""
        return self._settings

    @property
    def pa_mgr(self):
        """Get PulseAudioManager instance."""
        return self._pa_mgr
