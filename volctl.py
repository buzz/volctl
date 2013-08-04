#! /usr/bin/python

import os
import math
from subprocess import Popen
import gtk

from pa_mgr import PulseAudioManager

PROGRAM_NAME = "Volume Control"
VERSION = "0.1"
COPYRIGHT = "(c) Mirko Dietrich"
LICENSE = "GPLv2"
COMMENTS = "PulseAudio enabled volume control status icon."
WEBSITE = "www.github.com/buzz/volctl"

SCRIPTPATH = os.path.dirname(os.path.realpath(__file__))
IMAGEPATH = os.path.join(SCRIPTPATH, "images")
TRAY_ICONS = [
    os.path.join(IMAGEPATH, "audio-volume-muted.png"),
    os.path.join(IMAGEPATH, "audio-volume-low.png"),
    os.path.join(IMAGEPATH, "audio-volume-medium.png"),
    os.path.join(IMAGEPATH, "audio-volume-high.png"),
    ]
MIXER_CMD = "/usr/bin/pavucontrol"
# granularity of volume control
STEPS = 10

class VolCtlTray():

    def __init__(self):
        # connect to pulseaudio
        self.pa_mgr = PulseAudioManager(self.cb_pa_update)
        self.pa_mgr.connect()

        # status icon
        self.pixbufs = [gtk.gdk.pixbuf_new_from_file(f) for f in TRAY_ICONS]
        self.statusicon = gtk.StatusIcon()
        self.statusicon.set_title("Volume")
        self.statusicon.set_name("Volume")
        self.statusicon.set_from_pixbuf(self.pixbufs[0])
        self.statusicon.set_has_tooltip(True)
        self.statusicon.connect("popup-menu", self.cb_popup)
        self.statusicon.connect("button-press-event", self.cb_button_press)
        self.statusicon.connect("scroll-event", self.cb_scroll)
        self.statusicon.connect("query-tooltip", self.cb_tooltip)

        # popup menu
        self.menu = gtk.Menu()
        mute_menu_item = gtk.ImageMenuItem("Mute")
        img = gtk.Image()
        img.set_from_file(os.path.join(IMAGEPATH, "audio-volume-muted-16.png"))
        mute_menu_item.set_image(img)
        mixer_menu_item = gtk.ImageMenuItem("Mixer")
        img = gtk.Image()
        img.set_from_file(os.path.join(IMAGEPATH, "mixer.png"))
        mixer_menu_item.set_image(img)

        mute_menu_item.connect("activate", self.cb_mute)
        mixer_menu_item.connect("activate", self.cb_mixer)
        about_menu_item = gtk.ImageMenuItem(gtk.STOCK_ABOUT)
        about_menu_item.connect("activate", self.cb_about)
        exit_menu_item = gtk.ImageMenuItem(gtk.STOCK_QUIT)
        exit_menu_item.connect("activate", self.cb_quit)

        self.menu.append(mute_menu_item)
        self.menu.append(mixer_menu_item)
        self.menu.append(gtk.SeparatorMenuItem())
        self.menu.append(about_menu_item)
        self.menu.append(exit_menu_item)
        self.menu.show_all()

        self.slider = None

        self.update_icon(self.pa_mgr.get_volume())

    def cb_pa_update(self, vol, muted):
        if not muted is None:
            if muted:
                self.statusicon.set_from_pixbuf(self.pixbufs[0])
            else:
                self.update_icon(self.pa_mgr.get_volume())
            if not self.slider is None:
                self.slider.update_scale(self.slider.master_scale, None, muted)
        if not vol is None:
            self.update_icon(vol)
            if not self.slider is None:
                self.slider.update_scale(self.slider.master_scale, vol, None)

    def update_icon(self, vol, muted=None):
        if muted is None:
            muted = self.pa_mgr.get_mute()
        v = min(vol, 1)
        if v == 0 or muted:
            idx = 0
        else:
            idx = int((len(self.pixbufs) - 2) * v) + 1
        self.statusicon.set_from_pixbuf(self.pixbufs[idx])

    def launch_mixer(self):
        Popen(MIXER_CMD)

    def cb_tooltip(self,item, x, y, keyboard_mode, tooltip):
        text = "Volume: %i%%" % (self.pa_mgr.get_volume() * 100)
        if self.pa_mgr.get_mute():
            text += " <span weight='bold'>(muted)</span>"
        tooltip.set_markup(text)
        return True

    def cb_mute(self, widget):
        self.pa_mgr.toggle_mute()

    def cb_mixer(self, widget):
        self.launch_mixer()

    def cb_about(self, widget):
        about = gtk.AboutDialog()
        about.set_program_name(PROGRAM_NAME)
        about.set_version(VERSION)
        about.set_copyright(COPYRIGHT)
        about.set_license(LICENSE)
        about.set_comments(COMMENTS)
        about.set_website(WEBSITE)
        about.set_logo(
            gtk.gdk.pixbuf_new_from_file(
                os.path.join(IMAGEPATH, "audio-volume-high-128.png")))
        about.run()
        about.destroy()

    def cb_quit(self, widget):
        if gtk.main_level() > 0:
            gtk.main_quit()
        else:
            exit(1)

    def cb_scroll(self, widget, ev):
        if ev.direction == gtk.gdk.SCROLL_UP:
            self.pa_mgr.change_volume(1. / STEPS)
        elif ev.direction == gtk.gdk.SCROLL_DOWN:
            self.pa_mgr.change_volume(- 1. / STEPS)

    def cb_button_press(self, widget, ev):
        if ev.button == 1:
            if ev.type == gtk.gdk.BUTTON_PRESS:
                if self.slider is None:
                    self.slider = VolumeSlider(self)
                else:
                    self.slider.close()
                    self.slider = None
            if ev.type == gtk.gdk._2BUTTON_PRESS:
                if not self.slider is None:
                    self.slider.close()
                    self.slider = None
                self.launch_mixer()

    def cb_popup(self, icon, button, time):
        self.menu.popup(None, None, None, button, time)

class VolumeSlider:
    def __init__(self, volctl):
        self.statusicon = volctl.statusicon
        self.pa_mgr = volctl.pa_mgr
        self.clients = self.pa_mgr.get_clients()
        self.win = gtk.Window(type=gtk.WINDOW_POPUP)
        self.table = gtk.Table()

        # position
        screen, rect, orient = self.statusicon.get_geometry()
        x, y, self.w, h = rect
        self.win.move(x, y + h)

        self.scales = []

        # remember obj_path for each stream scale
        self.scale_streams = {}

        # add master
        self.master_scale = self.add_scale(
            "Master", self.cb_scale_master_value_change, "audio-card")
        self.update_scale(
            self.master_scale, self.pa_mgr.get_volume(), self.pa_mgr.get_mute())

        # add clients
        for c in self.clients:
            scale = self.add_scale(
                c.name, self.cb_scale_stream_value_change, c.icon_name)
            self.scale_streams[scale] = c.stream_obj_path
            self.update_scale(scale,
                self.pa_mgr.get_stream_volume(c.stream_obj_path),
                self.pa_mgr.get_stream_mute(c.stream_obj_path))
        self.win.add(self.table)
        self.win.show_all()

    def add_scale(self, name, cb, icon_name=None):
        scale = gtk.VScale()
        scale.set_update_policy(gtk.UPDATE_DELAYED)
        scale.set_draw_value(False)
        scale.set_digits(1)
        scale.set_range(0, 1.0)
        scale.set_inverted(True)
        scale.set_size_request(self.w, 120)
        scale.set_increments(0.05, 0.1)
        scale.set_tooltip_text(name)
        self.scales.append(scale)
        x = len(self.scales)
        self.table.attach(scale, x, x + 1, 0, 1)
        scale.connect("value-changed", cb)
        image = gtk.Image()
        image.set_tooltip_text(name)
        if icon_name is None:
            image.set_from_icon_name("audio-card", gtk.ICON_SIZE_SMALL_TOOLBAR)
        else:
            image.set_from_icon_name(icon_name, gtk.ICON_SIZE_SMALL_TOOLBAR)
        self.table.attach(image, x, x + 1, 1, 2)
        return scale

    def update_scale(self, scale, value, muted):
        try:
            if not value is None:
                scale.set_value(value)
                if not muted is None:
                    scale.set_sensitive(not muted)
        except KeyError:
            pass

    # callback for scale gui element update (master)
    def cb_scale_master_value_change(self, scale):
        value = scale.get_value()
        self.pa_mgr.set_volume(value)
        # update other scales as automatically are changed depending on master
        for s in self.scales:
            if s != scale:
                value = self.pa_mgr.get_stream_volume(self.scale_streams[s])
                s.set_value(value)

    # callback for scale gui element update (playback stream)
    def cb_scale_stream_value_change(self, scale):
        try:
            value = scale.get_value()
            # scale callback
            self.pa_mgr.set_stream_volume(self.scale_streams[scale], value)
        except KeyError:
            # stream not found, update master vol
            self.pa_mgr.set_volume(value)

    def close(self):
        self.win.destroy()

if __name__ == "__main__":
    gtk.gdk.threads_init()
    vctray = VolCtlTray()
    gtk.main()
