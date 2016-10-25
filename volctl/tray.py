import os
from math import floor
from subprocess import Popen
import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, Gdk, Gio

from lib_pulseaudio import PA_VOLUME_MUTED, PA_VOLUME_NORM, \
     pa_threaded_mainloop_lock, pa_threaded_mainloop_unlock
from pa_mgr import PulseAudioManager

from volctl._version import __version__
from volctl.prefs import PreferencesDialog
from volctl.slider import VolumeSlider


DEFAULT_MIXER_CMD = '/usr/bin/pavucontrol'

PROGRAM_NAME = 'Volume Control'
COPYRIGHT =  '(c) buzz'
LICENSE = Gtk.License.GPL_2_0
COMMENTS = 'Per-application volume control for GNU/Linux desktops'
WEBSITE = 'https://buzz.github.io/volctl/'

class VolCtlTray():

    def __init__(self):
        self.settings = Gio.Settings('apps.volctl', path='/apps/volctl/')
        self.settings.connect('changed', self.cb_settings_changed)
        self.mouse_wheel_step = self.settings.get_int('mouse-wheel-step')

        self.volume = 0
        self.mute = False

        # status icon
        self.statusicon = Gtk.StatusIcon()
        self.statusicon.set_title('Volume')
        self.statusicon.set_name('Volume')
        self.statusicon.set_has_tooltip(True)
        self.statusicon.connect('popup-menu', self.cb_popup)
        self.statusicon.connect('button-press-event', self.cb_button_press)
        self.statusicon.connect('scroll-event', self.cb_scroll)
        self.statusicon.connect('query-tooltip', self.cb_tooltip)

        # windows
        self.about = None
        self.preferences = None

        # popup menu
        self.menu = Gtk.Menu()
        mute_menu_item = Gtk.ImageMenuItem('Mute')
        img = Gtk.Image.new_from_icon_name(
            'audio-volume-muted', Gtk.IconSize.SMALL_TOOLBAR)
        mute_menu_item.set_image(img)
        mute_menu_item.connect('activate', self.cb_mute)

        mixer_menu_item = Gtk.ImageMenuItem('Mixer')
        img = Gtk.Image.new_from_icon_name(
            'multimedia-volume-control', Gtk.IconSize.SMALL_TOOLBAR)
        mixer_menu_item.set_image(img)
        mixer_menu_item.connect('activate', self.cb_mixer)

        preferences_menu_item = Gtk.ImageMenuItem('Preferences')
        img = Gtk.Image.new_from_icon_name(
            'preferences-desktop', Gtk.IconSize.SMALL_TOOLBAR)
        preferences_menu_item.set_image(img)
        preferences_menu_item.connect('activate', self.cb_preferences)

        about_menu_item = Gtk.ImageMenuItem.new_from_stock(Gtk.STOCK_ABOUT)
        about_menu_item.connect('activate', self.cb_about)

        exit_menu_item = Gtk.ImageMenuItem.new_from_stock(Gtk.STOCK_QUIT)
        exit_menu_item.connect('activate', self.cb_quit)

        self.menu.append(mute_menu_item)
        self.menu.append(mixer_menu_item)
        self.menu.append(preferences_menu_item)
        self.menu.append(Gtk.SeparatorMenuItem())
        self.menu.append(about_menu_item)
        self.menu.append(exit_menu_item)
        self.menu.show_all()

        self.pa_mgr = PulseAudioManager(self)

    def launch_mixer(self):
        mixer_cmd = self.settings.get_string('mixer-command')
        if mixer_cmd == '':
            mixer_cmd = DEFAULT_MIXER_CMD
        Popen(mixer_cmd)

    def update_icon(self):
        v = float(self.volume) / float(PA_VOLUME_NORM)
        if self.mute:
            state = 'muted'
        else:
            idx = min(int(floor(v * 3)), 2)
            state = ['low', 'medium', 'high'][idx]
        icon_name = 'audio-volume-%s' % state
        self.statusicon.set_from_icon_name(icon_name)

    def cb_tooltip(self,item, x, y, keyboard_mode, tooltip):
        perc = float(self.volume) / float(PA_VOLUME_NORM) * 100
        text = 'Volume: %.0f%%' % perc
        if self.mute:
            text += ' <span weight="bold">(muted)</span>'
        tooltip.set_markup(text)
        return True

    def cb_mute(self, widget):
        m = self.pa_mgr.pa.pa_mainloop
        pa_threaded_mainloop_lock(m)
        self.pa_mgr.toggle_mute()
        pa_threaded_mainloop_unlock(m)

    def cb_mixer(self, widget):
        self.launch_mixer()

    def cb_preferences(self, widget):
        try:
            self.preferences.present()
        except AttributeError:
            self.preferences = PreferencesDialog(self.settings)
            response = self.preferences.run()
            self.preferences.destroy()
            del self.preferences

    def cb_about(self, widget):
        try:
            self.about.present()
        except AttributeError:
            self.about = Gtk.AboutDialog()
            self.about.set_program_name(PROGRAM_NAME)
            self.about.set_version(__version__)
            self.about.set_copyright(COPYRIGHT)
            self.about.set_license_type(LICENSE)
            self.about.set_comments(COMMENTS)
            self.about.set_website(WEBSITE)
            self.about.set_logo_icon_name('audio-volume-high')
            self.about.run()
            self.about.destroy()
            del self.about

    def cb_quit(self, widget):
        if Gtk.main_level() > 0:
            try:
                self.preferences.response(0)
            except AttributeError:
                pass
            try:
                self.about.close()
            except AttributeError:
                pass
            Gtk.main_quit()
        else:
            exit(1)

    def cb_scroll(self, widget, ev):
        old_vol = self.volume
        amount = PA_VOLUME_NORM / self.settings.get_int('mouse-wheel-step')
        if ev.direction == Gdk.ScrollDirection.DOWN:
            amount *= -1
        elif ev.direction == Gdk.ScrollDirection.UP:
            pass
        else:
            return
        new_value = old_vol + amount
        new_value = min(PA_VOLUME_NORM, new_value)
        new_value = max(PA_VOLUME_MUTED, new_value)

        # user action prolongs auto-close timer
        if hasattr(self, 'slider'):
            self.slider.reset_timeout()

        m = self.pa_mgr.pa.pa_mainloop
        pa_threaded_mainloop_lock(m)
        self.pa_mgr.set_volume(new_value)
        pa_threaded_mainloop_unlock(m)

    def cb_button_press(self, widget, ev):
        if ev.button == 1:
            if ev.type == Gdk.EventType.BUTTON_PRESS:
                if hasattr(self, 'slider'):
                    self.close_slider()
                else:
                    self.show_slider()
            if ev.type == Gdk.EventType._2BUTTON_PRESS:
                self.launch_mixer()

    def show_slider(self):
        self.slider = VolumeSlider(self)

    def close_slider(self):
        self.slider.close()
        del self.slider

    def cb_popup(self, icon, button, time):
        self.menu.popup(None, None, None, None, button, time)

    # updates coming from pulse

    def sink_count_changed(self):
        if hasattr(self, 'slider'):
            self.close_slider()
            self.show_slider()

    def update_values(self, volume, mute):
        self.volume = volume
        self.mute = mute
        self.update_icon()

    def update_sink_scale(self, idx, volume, mute):
        if hasattr(self, 'slider'):
            self.slider.update_sink_scale(idx, volume, mute)

    def update_sink_input_scale(self, idx, volume, mute):
        if hasattr(self, 'slider'):
            self.slider.update_sink_input_scale(idx, volume, mute)

    # gsettings callback

    def cb_settings_changed(self, settings, key):
        if key == 'mouse-wheel-step':
            self.mouse_wheel_step = settings.get_int('mouse-wheel-step')
            if hasattr(self, 'slider'):
                self.slider.set_increments()
