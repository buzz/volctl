import os
import re
from sys import stdout
import gobject
from subprocess import Popen
import dbus
from dbus.mainloop.glib import DBusGMainLoop

APP_TO_ICON_NAME = {
    "chrome": "chromium-browser",
}

def arr2str(arr):
    # concatenate chars to string
    h = "".join([chr(c) for c in arr])
    # strip null byte
    h = h.rstrip(" \t\r\n\0")
    return h

class PulseAudioManager():

    def __init__(self, cb_update):
        # cb_update(vol, mute)
        self.cb_update = cb_update
        self.clients = None

    def connect(self):
        DBusGMainLoop(set_as_default=True)
        sbus = dbus.SessionBus()
        srv_addr = None
        srv_addr = os.environ.get("PULSE_DBUS_SERVER")
        if not srv_addr:
            srv_addr = sbus.get_object(
                "org.PulseAudio1", "/org/pulseaudio/server_lookup1")\
                .Get("org.PulseAudio.ServerLookup1", "Address",
                     dbus_interface="org.freedesktop.DBus.Properties")
        self.dbus = dbus.connection.Connection(srv_addr)
        self.core = self.dbus.get_object(object_path="/org/pulseaudio/core1")
        self.core_prop_mgr = dbus.Interface(
            self.core, "org.freedesktop.DBus.Properties")
        self.sink = self.dbus.get_object(
            object_path="/org/pulseaudio/core1/sink0")
        self.sink_prop_mgr = dbus.Interface(
            self.sink, "org.freedesktop.DBus.Properties")

        # connect signals (for master volume)
        for sig_name, sig_handler in (
            ("MuteUpdated", self._cb_mute_updated),
            ("VolumeUpdated", self._cb_volume_updated)):
            if not sig_handler is None:
                self.dbus.add_signal_receiver(sig_handler, sig_name)
                self.core.ListenForSignal(
                    "org.PulseAudio.Core1.Device.%s" %
                    sig_name, dbus.Array(signature="o"))

    def _cb_volume_updated(self, vol):
        self.cb_update(vol[0] / 65536., None)

    def _cb_mute_updated(self, state):
        self.cb_update(None, bool(state))

    def get_mute(self):
        return self.sink_prop_mgr.Get("org.PulseAudio.Core1.Device", "Mute")

    def set_mute(self, state):
        self.sink_prop_mgr.Set("org.PulseAudio.Core1.Device", "Mute", state)

    def toggle_mute(self):
        if self.get_mute():
            self.set_mute(False)
        else:
            self.set_mute(True)

    def get_volume(self):
        vol = self.sink_prop_mgr.Get("org.PulseAudio.Core1.Device", "Volume")
        return (vol[0] / 65536.)

    def set_volume(self, vol):
        vol = max(min((vol * 65536), 65536), 0)
        vol = self.sink_prop_mgr.Set("org.PulseAudio.Core1.Device", "Volume",
                                     [dbus.UInt32(vol)])

    def change_volume(self, change):
        self.set_volume(self.get_volume() + change)

    def get_stream_volume(self, obj_path):
            stream = self.dbus.get_object(object_path=obj_path)
            prop_mgr = dbus.Interface(stream, "org.freedesktop.DBus.Properties")
            return prop_mgr.Get("org.PulseAudio.Core1.Stream", "Volume")[0] / 65536.

    def set_stream_volume(self, obj_path, vol):
            vol = max(min((vol * 65536), 65536), 0)
            stream = self.dbus.get_object(object_path=obj_path)
            prop_mgr = dbus.Interface(stream, "org.freedesktop.DBus.Properties")
            vol = prop_mgr.Set("org.PulseAudio.Core1.Stream", "Volume",
                               [dbus.UInt32(vol)])

    def get_stream_mute(self, obj_path):
            stream = self.dbus.get_object(object_path=obj_path)
            prop_mgr = dbus.Interface(stream, "org.freedesktop.DBus.Properties")
            return bool(prop_mgr.Get("org.PulseAudio.Core1.Stream", "Mute"))

    def set_stream_mute(self, obj_path, state):
            stream = self.dbus.get_object(object_path=obj_path)
            prop_mgr = dbus.Interface(stream, "org.freedesktop.DBus.Properties")
            vol = prop_mgr.Set(
                "org.PulseAudio.Core1.Stream", "Mute", dbus.Bool(state))

    def get_clients(self):
        obj_paths = self.core_prop_mgr.Get("org.PulseAudio.Core1", "Clients")
        p = re.compile("^ALSA plug-in \[(.*)]")
        clients = []
        for obj_path in obj_paths:
            client = self.dbus.get_object(object_path=obj_path)
            prop_mgr = dbus.Interface(client, "org.freedesktop.DBus.Properties")
            idx = prop_mgr.Get("org.PulseAudio.Core1.Client", "Index")
            streams = prop_mgr.Get(
                "org.PulseAudio.Core1.Client", "PlaybackStreams")
            if len(streams) > 0:
                prop_list = prop_mgr.Get("org.PulseAudio.Core1.Client",
                                         "PropertyList")
                name = arr2str(prop_list["application.name"])

                # remove cruft from alsa streams
                m = p.match(name)
                if m:
                    name = m.group(1)

                try:
                    icon_name = arr2str(prop_list["application.icon_name"])
                except KeyError:
                    try:
                        icon_name = APP_TO_ICON_NAME[name]
                    except KeyError:
                        icon_name = None
                clients.append(Client(
                        idx, obj_path, name, streams[0], icon_name=icon_name))
        return clients

    def listen_for_stream_change(self, cb):
        self.dbus.add_signal_receiver(cb, "VolumeUpdated")
        self.core.ListenForSignal(
            "org.PulseAudio.Core1.Device.%s" %
            sig_name, dbus.Array(signature="o"))

class Client():
    def __init__(self, idx, obj_path, name, stream_obj_path, icon_name=None):
        self.idx = idx
        self.obj_path = obj_path
        self.name = name
        self.stream_obj_path = stream_obj_path
        self.icon_name = icon_name

    def __str__(self):
        return "PulseAudio Client (%i, %s, %s, %s, %s)" % (
            self.idx, self.obj_path, self.name, self.stream_obj_path,
            self.icon_name)
