"""volctl application"""

from subprocess import Popen
from gi.repository import Gio
from gi.repository import Gtk

from volctl.meta import (PROGRAM_NAME, COPYRIGHT, LICENSE, COMMENTS, WEBSITE,
                         VERSION)
from volctl.tray import TrayIcon
from volctl.pulseaudio import PulseAudioManager
from volctl.prefs import PreferencesDialog
from volctl.slider_win import VolumeSliders

DEFAULT_MIXER_CMD = '/usr/bin/pavucontrol'

# TODO: Mirror all settings in app class, other classes should not use settings
#       directly.


class VolctlApp():
    """GUI application for volctl."""

    def __init__(self):
        self._settings = Gio.Settings('apps.volctl', path='/apps/volctl/')
        self._settings.connect('changed', self._cb_settings_changed)
        self._mouse_wheel_step = self._settings.get_int('mouse-wheel-step')

        self._pa_mgr = PulseAudioManager(self)

        # GUI
        self._tray_icon = TrayIcon(self)
        self._sliders_win = None
        self._about_win = None
        self._preferences = None

    def quit(self):
        """Gracefully shut down application."""
        try:
            self._pa_mgr.close()
        except AttributeError:
            pass
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

    def slider_count_changed(self):
        """Amount of sliders changed."""
        try:
            self._close_slider()
            self._show_slider()
        except AttributeError:
            pass

    # updates coming from pulseaudio

    def update_values(self, volume, mute):
        """Main sink values are reflected in status icon."""
        self._tray_icon.update_values(volume, mute)

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

    # gsettings callback

    def _cb_settings_changed(self, settings, key):
        if key == 'mouse-wheel-step':
            self._mouse_wheel_step = settings.get_int('mouse-wheel-step')
            try:
                self._sliders_win.set_increments()
            except AttributeError:
                pass

    # GUI

    def show_preferences(self):
        """Bring preferences to focus or create if it doesn't exist."""
        try:
            self._preferences.present()
        except AttributeError:
            self._preferences = PreferencesDialog(self._settings)
            self._preferences.run()
            self._preferences.destroy()
            del self._preferences

    def show_about(self):
        """Bring about window to focus or create if it doesn't exist."""
        try:
            self._about_win.present()
        except AttributeError:
            self._about_win = Gtk.AboutDialog()
            self._about_win.set_program_name(PROGRAM_NAME)
            self._about_win.set_version(VERSION)
            self._about_win.set_copyright(COPYRIGHT)
            self._about_win.set_license_type(LICENSE)
            self._about_win.set_comments(COMMENTS)
            self._about_win.set_website(WEBSITE)
            self._about_win.set_logo_icon_name('audio-volume-high')
            self._about_win.run()
            self._about_win.destroy()
            del self._about_win

    def launch_mixer(self):
        """Launch external mixer."""
        mixer_cmd = self._settings.get_string('mixer-command')
        if mixer_cmd == '':
            mixer_cmd = DEFAULT_MIXER_CMD
        Popen(mixer_cmd)

    def show_slider(self):
        """Show mini window with application volume sliders."""
        self._sliders_win = VolumeSliders(self)

    def close_slider(self):
        """Close mini window with application volume sliders."""
        try:
            self._sliders_win.close()
            del self._sliders_win
            return True
        except AttributeError:
            return False

    # some stuff is exposed to the outside

    @property
    def mouse_wheel_step(self):
        """Get increment for one mouse wheel tick."""
        return self._mouse_wheel_step

    @property
    def statusicon_geometry(self):
        """Get status icon geometry."""
        return self._tray_icon.get_geometry()

    @property
    def settings(self):
        """Get GSettings instance."""
        return self._settings

    @property
    def pa_mgr(self):
        """Get PulseAudioManager instance."""
        return self._pa_mgr
