"""volctl application"""

from subprocess import Popen
import sys
from gi.repository import Gdk, Gio, Gtk

from volctl.meta import (
    PROGRAM_NAME,
    COPYRIGHT,
    LICENSE,
    COMMENTS,
    WEBSITE,
    VERSION,
)
from volctl.status_icon import StatusIcon

from volctl.osd import VolumeOverlay
from volctl.prefs import PreferencesDialog
from volctl.pulsemgr import PulseManager
from volctl.slider_win import VolumeSliders


DEFAULT_MIXER_CMD = "pavucontrol"

TOGGLE_BUTTON_CSS = b"""
button.toggle {
    padding: 0;
    margin-bottom: -5px;
}
button.toggle:hover {
    background-color: transparent;
    border-color: transparent;
}
button.toggle:checked {
    background-color: transparent;
    border-color: transparent;
}
button.toggle:checked image {
    -gtk-icon-effect: dim;
}
"""


class VolctlApp:
    """GUI application for volctl."""

    def __init__(self):
        self._set_style(Gtk.CssProvider())
        self.settings = Gio.Settings("apps.volctl", path="/apps/volctl/")
        self.settings.connect("changed", self._cb_settings_changed)
        self.mouse_wheel_step = self.settings.get_int("mouse-wheel-step")
        self._first_volume_update = True
        self.pulsemgr = PulseManager(self)

        self.status_icon = None
        self.sliders_win = None
        self._about_win = None
        self._preferences = None
        self._osd = None
        self._mixer_process = None

        # Remembered main volume, mute
        self._volume, self._mute = 0.0, False

    def create_status_icon(self):
        """Create status icon."""
        if self.status_icon is None:
            prefer_gtksi = self.settings.get_boolean("prefer-gtksi")
            self.status_icon = StatusIcon(self, prefer_gtksi)

    def quit(self):
        """Gracefully shut down application."""
        try:
            self.pulsemgr.close()
        except AttributeError:
            pass
        if Gtk.main_level() > 0:
            if self.sliders_win:
                self.sliders_win.destroy()
            if self._about_win:
                self._about_win.destroy()
            if self._preferences:
                self._preferences.destroy()
            if self._osd:
                self._osd.destroy()
            Gtk.main_quit()
        else:
            sys.exit(1)

    @staticmethod
    def _set_style(provider):
        provider.load_from_data(TOGGLE_BUTTON_CSS)
        Gtk.StyleContext.add_provider_for_screen(
            Gdk.Screen.get_default(),
            provider,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION,
        )

    def _create_osd(self):
        self._osd = VolumeOverlay(self)
        self._osd.connect("destroy", self.on_osd_destroy)

    def on_osd_destroy(self, _):
        """OSD window destroy callback."""
        self._osd.disconnect_by_func(self.on_osd_destroy)
        del self._osd
        self._osd = None

    def update_main(self, volume, mute):
        """Default sink update."""

        # Ignore events that don't change anything (prevents OSD from showing)
        if volume == self._volume and mute == self._mute:
            return
        self._volume = volume
        self._mute = mute

        self.status_icon.update(volume, mute)
        # OSD
        if self._first_volume_update:
            self._first_volume_update = False  # Avoid showing on program start
            return
        if self.settings.get_boolean("osd-enabled"):
            if self._osd is None:
                self._create_osd()
            self._osd.update_values(volume, mute)
        elif self._osd is not None:
            self._osd.destroy()

    # Updates coming from pulseaudio

    def sink_update(self, idx, volume, mute):
        """A sink update is coming from PulseAudio."""
        if idx == self.pulsemgr.default_sink_idx:
            self.update_main(volume, mute)
        if self.sliders_win:
            self.sliders_win.update_sink_scale(idx, volume, mute)

    def sink_input_update(self, idx, volume, mute):
        """A sink input update is coming from PulseAudio."""
        if self.sliders_win:
            self.sliders_win.update_sink_input_scale(idx, volume, mute)

    def peak_update(self, idx, val):
        """Notify scale when update is coming from pulseaudio."""
        if self.sliders_win:
            self.sliders_win.update_scale_peak(idx, val)

    def slider_count_changed(self):
        """Amount of sliders changed."""
        if self.status_icon and self.sliders_win:
            self.sliders_win.recreate_sliders()
            if self.settings.get_boolean("vu-enabled"):
                self.pulsemgr.start_peak_monitor()

    def on_connected(self):
        """PulseAudio connection was established."""
        self.create_status_icon()

    def on_disconnected(self):
        """PulseAudio connection was lost."""
        self.close_slider()

    # Gsettings callback

    def _cb_settings_changed(self, settings, key):
        if key == "mouse-wheel-step":
            self.mouse_wheel_step = settings.get_int("mouse-wheel-step")
            if self.sliders_win:
                self.sliders_win.set_increments()

    # GUI

    def show_preferences(self):
        """Bring preferences to focus or create if it doesn't exist."""
        if self._preferences:
            self._preferences.present()
        else:
            self._preferences = PreferencesDialog(self.settings, DEFAULT_MIXER_CMD)
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
        mixer_cmd_str = self.settings.get_string("mixer-command")
        if mixer_cmd_str == "":
            mixer_cmd = DEFAULT_MIXER_CMD
        else:
            mixer_cmd = mixer_cmd_str.rsplit(" ")
        if self._mixer_process is None or not self._mixer_process.poll() is None:
            self._mixer_process = Popen(mixer_cmd)
        # TODO: bring mixer win to front otherwise

    def show_slider(self, xpos, ypos):
        """Show mini window with application volume sliders."""
        self.sliders_win = VolumeSliders(self, xpos, ypos)
        if self.settings.get_boolean("vu-enabled"):
            self.pulsemgr.start_peak_monitor()

    def close_slider(self):
        """Close mini window with application volume sliders."""
        if self.sliders_win:
            self.sliders_win.destroy()
            del self.sliders_win
            self.sliders_win = None
            self.pulsemgr.stop_peak_monitor()
            return True
        return False
