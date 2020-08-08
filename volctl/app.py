"""volctl application"""

from subprocess import Popen
import sys
from gi.repository import Gio
from gi.repository import Gtk

from volctl.meta import (
    PROGRAM_NAME,
    COPYRIGHT,
    LICENSE,
    COMMENTS,
    WEBSITE,
    VERSION,
)
from volctl.tray import TrayIcon
from volctl.lib.pa_wrapper import PulseAudioManager
from volctl.prefs import PreferencesDialog
from volctl.slider_win import VolumeSliders
from volctl.volume_overlay import VolumeOverlay

DEFAULT_MIXER_CMD = "pavucontrol"


class VolctlApp:
    """GUI application for volctl."""

    def __init__(self):
        self._settings = Gio.Settings("apps.volctl", path="/apps/volctl/")
        self._settings.connect("changed", self._cb_settings_changed)
        self._mouse_wheel_step = self._settings.get_int("mouse-wheel-step")

        self._pa_mgr = PulseAudioManager(self)

        # GUI
        self._tray_icon = TrayIcon(self)
        self.sliders_win = None
        self._about_win = None
        self._preferences = None
        self._volume_overlay = None
        self._mixer_process = None

    def quit(self):
        """Gracefully shut down application."""
        try:
            self._pa_mgr.close()
        except AttributeError:
            pass
        if Gtk.main_level() > 0:
            if self.sliders_win:
                self.sliders_win.destroy()
            if self._about_win:
                self._about_win.destroy()
            if self._preferences:
                self._preferences.destroy()
            if self._volume_overlay:
                self._volume_overlay.destroy()
            Gtk.main_quit()
        else:
            sys.exit(1)

    def slider_count_changed(self):
        """Amount of sliders changed."""
        if self.sliders_win:
            self.close_slider()
            self.show_slider()

    def _create_volume_overlay(self):
        self._volume_overlay = VolumeOverlay(self)
        self._volume_overlay.connect("destroy", self.on_volume_overlay_destroy)

    def on_volume_overlay_destroy(self, _):
        self._volume_overlay.disconnect_by_func(self.on_volume_overlay_destroy)
        del self._volume_overlay
        self._volume_overlay = None

    # updates coming from pulseaudio

    def update_values(self, volume, mute):
        """Main sink update."""
        self._tray_icon.update_values(volume, mute)
        if self._settings.get_boolean("osd-enabled"):
            if self._volume_overlay is None:
                self._create_volume_overlay()
            self._volume_overlay.update_values(volume, mute)
        elif self._volume_overlay is not None:
            self._volume_overlay.destroy()

    def update_sink_scale(self, idx, volume, mute):
        """Notify sink scale if update is coming from pulseaudio."""
        if self.sliders_win:
            self.sliders_win.update_sink_scale(idx, volume, mute)

    def update_sink_input_scale(self, idx, volume, mute):
        """Notify sink input scale if update is coming from pulseaudio."""
        if self.sliders_win:
            self.sliders_win.update_sink_input_scale(idx, volume, mute)

    # gsettings callback

    def _cb_settings_changed(self, settings, key):
        if key == "mouse-wheel-step":
            self._mouse_wheel_step = settings.get_int("mouse-wheel-step")
            if self.sliders_win:
                self.sliders_win.set_increments()

    # GUI

    def show_preferences(self):
        """Bring preferences to focus or create if it doesn't exist."""
        if self._preferences:
            self._preferences.present()
        else:
            self._preferences = PreferencesDialog(self._settings)
            self._preferences.run()
            self._preferences.destroy()
            del self._preferences
            self._preferences = None

    def show_about(self):
        """Bring about window to focus or create if it doesn't exist."""
        if self._about_win is not None:
            self._about_win.present()
        else:
            self._about_win = Gtk.AboutDialog()
            self._about_win.set_program_name(PROGRAM_NAME)
            self._about_win.set_version(VERSION)
            self._about_win.set_copyright(COPYRIGHT)
            self._about_win.set_license_type(LICENSE)
            self._about_win.set_comments(COMMENTS)
            self._about_win.set_website(WEBSITE)
            self._about_win.set_logo_icon_name("audio-volume-high")
            self._about_win.run()
            self._about_win.destroy()
            del self._about_win
            self._about_win = None

    def launch_mixer(self):
        """Launch external mixer."""
        mixer_cmd = self._settings.get_string("mixer-command")
        if mixer_cmd == "":
            mixer_cmd = DEFAULT_MIXER_CMD
        if self._mixer_process is None or not self._mixer_process.poll() is None:
            self._mixer_process = Popen(mixer_cmd)

    def show_slider(self):
        """Show mini window with application volume sliders."""
        self.sliders_win = VolumeSliders(self)

    def close_slider(self):
        """Close mini window with application volume sliders."""
        if self.sliders_win:
            self.sliders_win.destroy()
            del self.sliders_win
            self.sliders_win = None
            return True
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
