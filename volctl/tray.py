import os
from math import floor
from subprocess import Popen
import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, Gdk, Gio, GLib

from lib_pulseaudio import PA_VOLUME_MUTED, PA_VOLUME_NORM, \
     pa_threaded_mainloop_lock, pa_threaded_mainloop_unlock
from pa_mgr import PulseAudioManager

from volctl._version import __version__
from volctl.prefs import PreferencesDialog


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
            if response == Gtk.ResponseType.OK:
                print("The OK button was clicked")
            elif response == Gtk.ResponseType.CANCEL:
                print("The Cancel button was clicked")
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

class VolumeSlider:
    def __init__(self, volctl):
        self.volctl = volctl
        self.win = Gtk.Window(type=Gtk.WindowType.POPUP)
        self.win.connect('enter-notify-event', self.cb_enter_notify)
        self.win.connect('leave-notify-event', self.cb_leave_notify)
        self.grid = Gtk.Grid()
        self.grid.set_column_spacing(2)
        self.grid.set_row_spacing(6)
        self.frame = Gtk.Frame()
        self.frame.set_shadow_type(Gtk.ShadowType.OUT)
        self.frame.add(self.grid)
        self.win.add(self.frame)

        # gui objects by index
        self.sink_scales = {}
        self.sink_input_scales = {}

        self.create_sliders()
        self.win.show_all()
        self.set_position()

        # timeout
        self.auto_close_timeout = None

    def _find_idx_by_scale(self, scale, scales):
        for idx, v in scales.iteritems():
            if scale == v:
                return idx
        # should never happen
        raise Exception('Sink index not found for scale!')

    def _find_sink_idx_by_scale(self, scale):
        return self._find_idx_by_scale(scale, self.sink_scales)

    def _find_sink_input_idx_by_scale(self, scale):
        return self._find_idx_by_scale(scale, self.sink_input_scales)

    def set_position(self):
        a, screen, rect, orient = self.volctl.statusicon.get_geometry()
        win_width, win_height = self.win.get_size()
        # slider window should not leave screen boundaries
        x = rect.x
        if x + win_width > screen.width():
            x = screen.width() - win_width
        # top or bottom panel?
        if rect.y > screen.height() / 2:
            self.win.move(x, rect.y - win_height)
        else:
            self.win.move(x, rect.y + rect.height)

    def create_sliders(self):
        x = 0

        # touching pa objects here!
        pa_threaded_mainloop_lock(self.volctl.pa_mgr.pa.pa_mainloop)

        # sinks
        for idx, sink in self.volctl.pa_mgr.pa_sinks.iteritems():
            scale, icon = self.add_scale(sink)
            self.sink_scales[sink.idx] = scale
            scale.connect('value-changed', self.cb_sink_scale)
            self.update_scale(scale, sink.volume, sink.mute)
            scale.set_margin_top(6)
            icon.set_margin_bottom(6)
            self.grid.attach(scale, x, 0, 1, 1)
            self.grid.attach(icon, x, 1, 1, 1)
            x += 1

        # separator
        if len(self.volctl.pa_mgr.pa_sink_inputs) > 0:
            separator = Gtk.Separator().new(Gtk.Orientation.VERTICAL)
            separator.set_margin_top(6)
            separator.set_margin_bottom(6)
            self.grid.attach(separator, x, 0, 1, 2)
            x += 1

        # sink inputs
        for idx, sink_input in self.volctl.pa_mgr.pa_sink_inputs.iteritems():
            scale, icon = self.add_scale(sink_input)
            self.sink_input_scales[sink_input.idx] = scale
            scale.connect('value-changed', self.cb_sink_input_scale)
            self.update_scale(scale, sink_input.volume, sink_input.mute)
            scale.set_margin_top(6)
            icon.set_margin_bottom(6)
            self.grid.attach(scale, x, 0, 1, 1)
            self.grid.attach(icon, x, 1, 1, 1)
            x += 1

        pa_threaded_mainloop_unlock(self.volctl.pa_mgr.pa.pa_mainloop)

    def add_scale(self, sink):
        # scale
        scale = Gtk.Scale().new(Gtk.Orientation.VERTICAL)
        scale.set_draw_value(False)
        scale.set_value_pos(Gtk.PositionType.BOTTOM)
        scale.set_range(PA_VOLUME_MUTED, PA_VOLUME_NORM)
        scale.set_inverted(True)
        scale.set_size_request(24, 128)
        scale.set_tooltip_text(sink.name)
        self._set_increments_on_scale(scale)

        # icon
        icon = Gtk.Image()
        icon.set_tooltip_text(sink.name)
        icon.set_from_icon_name(sink.icon_name, Gtk.IconSize.SMALL_TOOLBAR)

        return scale, icon

    def set_increments(self):
        for idx, scale in self.sink_scales.iteritems():
            self._set_increments_on_scale(scale)
        for idx, scale in self.sink_input_scales.iteritems():
            self._set_increments_on_scale(scale)

    def _set_increments_on_scale(self, scale):
        scale.set_increments(PA_VOLUME_NORM / self.volctl.mouse_wheel_step,
                             PA_VOLUME_NORM / self.volctl.mouse_wheel_step)

    def update_scale(self, scale, volume, mute):
        scale.set_value(volume)
        if not mute is None:
            scale.set_sensitive(not mute)

    # called by pa thread

    def update_sink_scale(self, idx, volume, mute):
        try:
            scale = self.sink_scales[idx]
        except KeyError:
            return
        self.update_scale(scale, volume, mute)

    def update_sink_input_scale(self, idx, volume, mute):
        try:
            scale = self.sink_input_scales[idx]
        except KeyError:
            return
        self.update_scale(scale, volume, mute)

    # gui callbacks

    def cb_sink_scale(self, scale):
        value = int(scale.get_value())
        idx = self._find_sink_idx_by_scale(scale)

        m = self.volctl.pa_mgr.pa.pa_mainloop
        pa_threaded_mainloop_lock(m)
        sink = self.volctl.pa_mgr.pa_sinks[idx]
        sink.set_volume(value)
        pa_threaded_mainloop_unlock(m)

    def cb_sink_input_scale(self, scale):
        value = int(scale.get_value())
        idx = self._find_sink_input_idx_by_scale(scale)

        m = self.volctl.pa_mgr.pa.pa_mainloop
        pa_threaded_mainloop_lock(m)
        sink_input = self.volctl.pa_mgr.pa_sink_inputs[idx]
        sink_input.set_volume(value)
        pa_threaded_mainloop_unlock(m)

    def cb_enter_notify(self, win, obj):
        print(win, obj)
        print('enter')

    def cb_leave_notify(self, win, obj):
        print(win, obj)
        print('leave')
        self.auto_close_timeout = GLib.timeout_add(
            GLib.PRIORITY_DEFAULT, 5000, self.close)

    def close(self):
        self.win.destroy()
