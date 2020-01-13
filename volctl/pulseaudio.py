"""
PulseAudio manager.

Interacts with auto-generated lib_pulseaudio ctypes bindings.
"""

from __future__ import print_function
import sys
from gi.repository import GObject
from .lib_pulseaudio import (
    # types
    pa_cvolume, pa_volume_t, pa_subscription_mask_t,
    pa_sink_info_cb_t, pa_context_notify_cb_t,
    pa_context_subscribe_cb_t, pa_client_info_cb_t, pa_sink_input_info_cb_t,
    pa_context_success_cb_t,
    # mainloop
    pa_threaded_mainloop_new, pa_threaded_mainloop_get_api,
    pa_threaded_mainloop_start, pa_threaded_mainloop_signal,
    # context
    pa_context_new, pa_context_connect, pa_context_disconnect,
    pa_context_set_state_callback,
    pa_context_subscribe, pa_context_set_subscribe_callback,
    pa_context_get_state, pa_context_get_client_info_list,
    pa_context_get_sink_info_list, pa_context_get_sink_input_info_list,
    pa_context_get_client_info, pa_context_get_sink_info_by_index,
    pa_context_get_sink_input_info, pa_context_set_sink_volume_by_index,
    pa_context_set_sink_mute_by_index, pa_context_set_sink_input_volume,
    # misc
    pa_operation_unref, pa_proplist_to_string,
    # constants
    PA_CONTEXT_READY, PA_SUBSCRIPTION_MASK_SINK,
    PA_SUBSCRIPTION_MASK_SINK_INPUT, PA_SUBSCRIPTION_MASK_CLIENT,
    PA_CONTEXT_FAILED, PA_CONTEXT_TERMINATED,
    PA_SUBSCRIPTION_EVENT_FACILITY_MASK, PA_SUBSCRIPTION_EVENT_CLIENT,
    PA_SUBSCRIPTION_EVENT_REMOVE, PA_SUBSCRIPTION_EVENT_SINK,
    PA_SUBSCRIPTION_EVENT_TYPE_MASK, PA_SUBSCRIPTION_EVENT_SINK_INPUT,
)


def cvolume_from_volume(volume, channels):
    """Convert single-value volume to PA cvolume."""
    cvolume = pa_cvolume()
    cvolume.channels = channels
    vol = pa_volume_t * 32
    cvolume.values = vol()
    for i in range(0, channels):
        cvolume.values[i] = volume
    return cvolume


class PulseAudio():
    """Handles connection to PA. Sets up callbacks."""
    # pylint: disable=too-many-instance-attributes

    def __init__(self, new_client_cb, remove_client_cb, new_sink_cb,
                 remove_sink_cb, new_sink_input_cb, remove_sink_input_cb):
        # pylint: disable=too-many-arguments

        self.new_client_cb = new_client_cb
        self.new_sink_input_cb = new_sink_input_cb
        self.remove_sink_input_cb = remove_sink_input_cb
        self.remove_client_cb = remove_client_cb
        self.new_sink_cb = new_sink_cb
        self.remove_sink_cb = remove_sink_cb

        self.pa_mainloop = pa_threaded_mainloop_new()
        self.pa_mainloop_api = pa_threaded_mainloop_get_api(self.pa_mainloop)

        self._context = pa_context_new(
            self.pa_mainloop_api, 'volctl'.encode('utf-8'))
        self.__context_notify_cb = pa_context_notify_cb_t(
            self._context_notify_cb)
        pa_context_set_state_callback(
            self._context, self.__context_notify_cb, None)
        pa_context_connect(self._context, None, 0, None)

        # create callbacks
        self.__null_cb = pa_context_success_cb_t(self._null_cb)
        self.__pa_sink_info_cb = pa_sink_info_cb_t(self._pa_sink_info_cb)
        self.__pa_context_subscribe_cb = pa_context_subscribe_cb_t(
            self._pa_context_subscribe_cb)
        self.__pa_sink_input_info_list_cb = pa_sink_input_info_cb_t(
            self._pa_sink_input_info_cb)
        self.__pa_client_info_list_cb = pa_client_info_cb_t(
            self._pa_client_info_cb)

        pa_threaded_mainloop_start(self.pa_mainloop)

    def set_sink_volume(self, index, cvolume):
        """Set volume for a sink by index."""
        operation = pa_context_set_sink_volume_by_index(
            self._context, index, cvolume, self.__null_cb, None)
        pa_operation_unref(operation)

    def set_sink_mute(self, index, mute):
        """Set mute for a sink by index."""
        operation = pa_context_set_sink_mute_by_index(
            self._context, index, mute, self.__null_cb, None)
        pa_operation_unref(operation)

    def set_sink_input_volume(self, index, cvolume):
        """Set mute for a sink input by index."""
        operation = pa_context_set_sink_input_volume(
            self._context, index, cvolume, self.__null_cb, None)
        pa_operation_unref(operation)

    def disconnect(self):
        """Terminate connection to PA."""
        pa_context_disconnect(self._context)

    def _context_notify_cb(self, context, userdata):
        state = pa_context_get_state(context)

        if state == PA_CONTEXT_READY:
            self._request_update()

            pa_context_set_subscribe_callback(
                self._context, self.__pa_context_subscribe_cb, None)
            submask = (pa_subscription_mask_t)(
                PA_SUBSCRIPTION_MASK_SINK |
                PA_SUBSCRIPTION_MASK_SINK_INPUT |
                PA_SUBSCRIPTION_MASK_CLIENT)
            operation = pa_context_subscribe(
                self._context,
                submask,
                self.__null_cb,
                None
            )
            pa_operation_unref(operation)
            print('PulseAudio: Connection ready', file=sys.stderr)

        elif state == PA_CONTEXT_FAILED:
            print('PulseAudio: Connection failed', file=sys.stderr)
            pa_threaded_mainloop_signal(self.pa_mainloop, 0)
            sys.exit(1)

        elif state == PA_CONTEXT_TERMINATED:
            print('PulseAudio: Connection terminated', file=sys.stderr)
            pa_threaded_mainloop_signal(self.pa_mainloop, 0)

    def _request_update(self):
        operation = pa_context_get_client_info_list(
            self._context, self.__pa_client_info_list_cb, None)
        pa_operation_unref(operation)

        operation = pa_context_get_sink_info_list(
            self._context, self.__pa_sink_info_cb, None)
        pa_operation_unref(operation)

        operation = pa_context_get_sink_input_info_list(
            self._context, self.__pa_sink_input_info_list_cb, True)
        pa_operation_unref(operation)

    def _pa_context_subscribe_cb(self, context, event_type, index, user_data):
        efac = event_type & PA_SUBSCRIPTION_EVENT_FACILITY_MASK
        etype = event_type & PA_SUBSCRIPTION_EVENT_TYPE_MASK
        if efac == PA_SUBSCRIPTION_EVENT_CLIENT:
            if etype == PA_SUBSCRIPTION_EVENT_REMOVE:
                self.remove_client_cb(int(index))
            else:
                operation = pa_context_get_client_info(
                    self._context, index, self.__pa_client_info_list_cb,
                    None)
                pa_operation_unref(operation)

        elif efac == PA_SUBSCRIPTION_EVENT_SINK:
            if etype == PA_SUBSCRIPTION_EVENT_REMOVE:
                self.remove_sink_cb(int(index))
            else:
                operation = pa_context_get_sink_info_by_index(
                    self._context, int(index), self.__pa_sink_info_cb, True)
                pa_operation_unref(operation)

        elif efac == PA_SUBSCRIPTION_EVENT_SINK_INPUT:
            if etype == PA_SUBSCRIPTION_EVENT_REMOVE:
                self.remove_sink_input_cb(int(index))
            else:
                operation = pa_context_get_sink_input_info(
                    self._context, int(index),
                    self.__pa_sink_input_info_list_cb, True)
                pa_operation_unref(operation)

    def _pa_client_info_cb(self, context, struct, c_int, user_data):
        if struct:
            self.new_client_cb(
                struct.contents.index, struct.contents,
                self._dict_from_proplist(struct.contents.proplist))

    def _pa_sink_input_info_cb(self, context, struct, index, user_data):
        if struct and user_data:
            self.new_sink_input_cb(int(struct.contents.index), struct.contents)

    def _pa_sink_info_cb(self, context, struct, index, data):
        if struct:
            self.new_sink_cb(
                int(struct.contents.index), struct.contents,
                self._dict_from_proplist(struct.contents.proplist))

    @staticmethod
    def _null_cb(param_a=None, param_b=None, param_c=None, param_d=None):
        return

    @staticmethod
    def _dict_from_proplist(proplist):
        props = {}
        proplist = pa_proplist_to_string(proplist).split('\n'.encode('utf-8'))
        for prop in proplist:
            left, _, right = prop.partition('='.encode('utf-8'))
            props[left.strip()] = right.strip()[1:-1]
        return props


class PulseAudioManager():
    """
    Main PulseAudio interface.

    Provides methods to UI. Internally uses PulseAudio object. Keeps track of
    connected clients, sinks, sink inputs.
    """

    def __init__(self, volctl):
        self.volctl = volctl
        self._pa_clients = {}
        self._pa_sinks = {}
        self._pa_sink_inputs = {}
        self._pulseaudio = PulseAudio(
            self._on_new_pa_client,
            self._on_remove_pa_client,
            self._on_new_pa_sink,
            self._on_remove_pa_sink,
            self._on_new_pa_sink_input,
            self._on_remove_pa_sink_input
        )

    @property
    def mainloop(self):
        """Get PulseAudio mainloop."""
        return self._pulseaudio.pa_mainloop

    @property
    def pa_sinks(self):
        """Get PulseAudio sinks."""
        return self._pa_sinks

    @property
    def pa_sink_inputs(self):
        """Get PulseAudio sink inputs."""
        return self._pa_sink_inputs

    def get_pa_client(self, client):
        """Return PulseAudio client."""
        return self._pa_clients[client]

    def close(self):
        """Close PA manager."""
        self._pulseaudio.disconnect()

    # called by Sink, SinkInput objects

    def get_first_sink(self):
        """Returns first sink (master volume)"""
        try:
            first_key = list(self._pa_sinks.keys())[0]
            return self._pa_sinks[first_key]
        except IndexError:
            pass
        return None

    def set_sink_volume(self, index, cvolume):
        """Set sink volume by index."""
        self._pulseaudio.set_sink_volume(index, cvolume)

    def set_sink_mute(self, index, mute):
        """Set sink mute by index."""
        self._pulseaudio.set_sink_mute(index, mute)

    def set_sink_input_volume(self, index, cvolume):
        """Set sink input volume by index."""
        self._pulseaudio.set_sink_input_volume(index, cvolume)

    # called by gui thread -> lock pa thread

    def set_main_volume(self, volume):
        """Set main volume"""
        self.get_first_sink().set_volume(volume)

    def toggle_main_mute(self):
        """Toggle main mute"""
        sink = self.get_first_sink()
        sink.set_mute(not sink.mute)

    # callbacks called by pulseaudio

    def _on_new_pa_client(self, index, struct, props):
        if index not in self._pa_clients:
            self._pa_clients[index] = Client(self, index)
        self._pa_clients[index].update(struct, props)

    def _on_remove_pa_client(self, index):
        if index in self._pa_clients:
            del self._pa_clients[index]

    def _on_new_pa_sink(self, index, struct, _):
        if index not in self._pa_sinks:
            self._pa_sinks[index] = Sink(self, index)
            # notify gui thread
            GObject.idle_add(self.volctl.slider_count_changed)
        self._pa_sinks[index].update(struct)

    def _on_remove_pa_sink(self, index):
        del self._pa_sinks[index]
        # notify gui thread
        GObject.idle_add(self.volctl.slider_count_changed)

    def _on_new_pa_sink_input(self, index, struct):
        # filter out strange events
        if struct.name == 'audio-volume-change':
            return
        # unknown if this is the right way to filter for applications
        # but seems to keep away things like loopback module etc.
        if 'protocol-native' not in struct.driver.decode('utf-8'):
            return
        if index not in self._pa_sink_inputs:
            self._pa_sink_inputs[index] = SinkInput(self, index)
            # notify gui thread
            GObject.idle_add(self.volctl.slider_count_changed)
        self._pa_sink_inputs[index].update(struct)

    def _on_remove_pa_sink_input(self, index):
        if index in self._pa_sink_inputs:
            del self._pa_sink_inputs[index]
            # notify gui thread
            GObject.idle_add(self.volctl.slider_count_changed)


class Sink():
    """An audio interface."""

    icon_name = 'audio-card'

    def __init__(self, pa_mgr, idx):
        self.pa_mgr = pa_mgr
        self.idx = idx
        self.scale = None
        self.name = ''
        self.volume = 0
        self.channels = 0
        self.mute = False

    def update(self, struct):
        """Update sink values."""
        # set values
        self.name = struct.description.decode('utf-8')
        self.volume = struct.volume.values[0]
        self.channels = struct.volume.channels
        self.mute = bool(struct.mute)
        # tray icon update (first sound card)
        if self == self.pa_mgr.get_first_sink():
            GObject.idle_add(
                self.pa_mgr.volctl.update_values, self.volume, self.mute)
        # scale update
        GObject.idle_add(self.pa_mgr.volctl.update_sink_scale, self.idx,
                         self.volume, self.mute)

    def set_volume(self, volume):
        """Set volume for this sink."""
        self.volume = volume
        cvolume = cvolume_from_volume(volume, self.channels)
        self.pa_mgr.set_sink_volume(self.idx, cvolume)

    def set_mute(self, mute):
        """Set mute for this sink."""
        self.mute = mute
        self.pa_mgr.set_sink_mute(self.idx, mute and 1 or 0)


class SinkInput():
    """An audio stream coming from a client."""

    def __init__(self, pa_mgr, idx):
        self.pa_mgr = pa_mgr
        self.idx = idx
        self.scale = None
        self.volume = 0
        self.channels = 0
        self.mute = False
        self.client = None

    def update(self, struct):
        """Update sink input values."""
        # set values
        self.volume = struct.volume.values[0]
        self.channels = struct.volume.channels
        self.mute = bool(struct.mute)
        self.client = struct.client
        # scale update
        GObject.idle_add(self.pa_mgr.volctl.update_sink_input_scale, self.idx,
                         self.volume, self.mute)

    def _get_client(self):
        return self.pa_mgr.get_pa_client(self.client)

    def set_volume(self, volume):
        """Set volume for this sink input."""
        self.volume = volume
        cvolume = cvolume_from_volume(volume, self.channels)
        self.pa_mgr.set_sink_input_volume(self.idx, cvolume)

    @property
    def icon_name(self):
        """Sink input icon name"""
        return self._get_client().icon_name

    @property
    def name(self):
        """Sink input name"""
        return self._get_client().name


class Client():
    """Represents an audio emitting application connected to PA."""

    # pylint: disable=too-few-public-methods

    def __init__(self, pa_mgr, idx):
        self.pa_mgr = pa_mgr
        self.idx = idx
        self.name = ''
        self.icon_name = None

    def update(self, struct, props):
        """Update client name and icon."""
        self.name = struct.name.decode('utf-8')
        self.icon_name = props.get(b'application.icon_name', b'multimedia-volume-control').decode('utf-8')
