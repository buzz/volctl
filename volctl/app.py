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
from volctl.lib.pulseaudio import pa_threaded_mainloop_lock, pa_threaded_mainloop_unlock
from volctl.prefs import PreferencesDialog
from volctl.slider_win import VolumeSliders
from volctl.osd import VolumeOverlay


DEFAULT_MIXER_CMD = "pavucontrol"


class VolctlApp:
    """GUI application for volctl."""

    def __init__(self):
        self.settings = Gio.Settings("apps.volctl", path="/apps/volctl/")
        self.settings.connect("changed", self._cb_settings_changed)
        self.mouse_wheel_step = self.settings.get_int("mouse-wheel-step")
        self._first_volume_update = True
        self._volume = 0
        self._mute = False

        self.pa_mgr = PulseAudioManager(self)

        # GUI
        self.tray_icon = TrayIcon(self)
        self.sliders_win = None
        self._about_win = None
        self._preferences = None
        self._osd = None
        self._mixer_process = None

    def quit(self):
        """Gracefully shut down application."""
        try:
            self.pa_mgr.close()
        except AttributeError:
            pass
        if Gtk.main_level() > 0:
            self.stop_vu()
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

    def _create_osd(self):
        self._osd = VolumeOverlay(self)
        self._osd.connect("destroy", self.on_osd_destroy)

    def on_osd_destroy(self, _):
        """OSD window destroy callback."""
        self._osd.disconnect_by_func(self.on_osd_destroy)
        del self._osd
        self._osd = None

    def start_vu(self):
        if self.settings.get_boolean("vu-enabled"):
            pa_threaded_mainloop_lock(self.pa_mgr.mainloop)
            for _, sink in self.pa_mgr.pa_sinks.items():
                sink.monitor_stream()
            for _, sink_input in self.pa_mgr.pa_sink_inputs.items():
                sink_input.monitor_stream()
            pa_threaded_mainloop_unlock(self.pa_mgr.mainloop)

    def stop_vu(self):
        pa_threaded_mainloop_lock(self.pa_mgr.mainloop)
        for _, sink in self.pa_mgr.pa_sinks.items():
            sink.stop_monitor_stream()
        for _, sink_input in self.pa_mgr.pa_sink_inputs.items():
            sink_input.stop_monitor_stream()
        pa_threaded_mainloop_unlock(self.pa_mgr.mainloop)

    # updates coming from pulseaudio

    def update_values(self, volume, mute):
        """Main sink update."""
        # no need to update if values didn't change
        if volume == self._volume and mute == self._mute:
            return

        self._volume = volume
        self._mute = mute

        # tray icon
        self.tray_icon.update_values(volume, mute)

        # OSD
        if self._first_volume_update:
            # Avoid showing on program start
            self._first_volume_update = False
            return
        if self.settings.get_boolean("osd-enabled"):
            if self._osd is None:
                self._create_osd()
            self._osd.update_values(volume, mute)
        elif self._osd is not None:
            self._osd.destroy()

    def update_sink_scale(self, idx, volume, mute):
        """Notify sink scale when update is coming from pulseaudio."""
        if self.sliders_win:
            self.sliders_win.update_sink_scale(idx, volume, mute)

    def update_sink_input_scale(self, idx, volume, mute):
        """Notify sink input scale when update is coming from pulseaudio."""
        if self.sliders_win:
            self.sliders_win.update_sink_input_scale(idx, volume, mute)

    def update_sink_peak(self, idx, val):
        """Notify sink scale when update is coming from pulseaudio."""
        if self.sliders_win:
            self.sliders_win.update_sink_scale_peak(idx, val)

    def update_sink_input_peak(self, idx, val):
        """Notify sink input scale when update is coming from pulseaudio."""
        if self.sliders_win:
            self.sliders_win.update_sink_input_scale_peak(idx, val)

    def slider_count_changed(self):
        """Amount of sliders changed."""
        if self.tray_icon and self.tray_icon.initialized and self.sliders_win:
            self.sliders_win.create_sliders()
            self.start_vu()

    # gsettings callback

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
        mixer_cmd = self.settings.get_string("mixer-command")
        if mixer_cmd == "":
            mixer_cmd = DEFAULT_MIXER_CMD
        if self._mixer_process is None or not self._mixer_process.poll() is None:
            self._mixer_process = Popen(mixer_cmd)
        # TODO: bring mixer win to front otherwise

    def show_slider(self, monitor_rect):
        """Show mini window with application volume sliders."""
        self.sliders_win = VolumeSliders(self, monitor_rect)
        self.start_vu()

    def close_slider(self):
        """Close mini window with application volume sliders."""
        if self.sliders_win:
            self.sliders_win.destroy()
            del self.sliders_win
            self.sliders_win = None
            self.stop_vu()
            return True
        return False
