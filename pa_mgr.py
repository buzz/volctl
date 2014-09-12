from pulseaudio.lib_pulseaudio import *
import sys
import ctypes
import gobject


APP_TO_ICON_NAME = {
    'Chromium': 'chromium-browser',
}

def cvolume_from_volume(volume, channels):
    cvolume = pa_cvolume()
    cvolume.channels = channels
    v = pa_volume_t * 32
    cvolume.values = v()

    for i in range(0, channels):
        cvolume.values[i] = volume
    return cvolume

def null_cb(a=None, b=None, c=None, d=None):
    return

class PulseAudio():
    def __init__(self, new_client_cb, remove_client_cb, new_sink_cb, remove_sink_cb, new_sink_input_cb, remove_sink_input_cb):

        self.new_client_cb = new_client_cb
        self.new_sink_input_cb = new_sink_input_cb
        self.remove_sink_input_cb = remove_sink_input_cb
        self.remove_client_cb = remove_client_cb
        self.new_sink_cb = new_sink_cb
        self.remove_sink_cb = remove_sink_cb

        self.pa_mainloop = pa_threaded_mainloop_new()
        self.pa_mainloop_api = pa_threaded_mainloop_get_api(self.pa_mainloop)

        self._context = pa_context_new(self.pa_mainloop_api, 'volctl')
        self._context_notify_cb = pa_context_notify_cb_t(self.context_notify_cb)
        pa_context_set_state_callback(
            self._context, self._context_notify_cb, None)
        pa_context_connect(self._context, None, 0, None);

        pa_threaded_mainloop_start(self.pa_mainloop)

    def disconnect(self):
        pa_context_disconnect(self._context)

    # pulseaudio connection status
    def context_notify_cb(self, context, userdata):
        ctc = pa_context_get_state(context)
        if ctc == PA_CONTEXT_READY:
            self._null_cb = pa_context_success_cb_t(null_cb)
            self._pa_context_success_cb = pa_context_success_cb_t(
                self.pa_context_success_cb)
            self._pa_sink_info_cb = pa_sink_info_cb_t(self.pa_sink_info_cb)
            self._pa_context_subscribe_cb = pa_context_subscribe_cb_t(
                self.pa_context_subscribe_cb)
            self._pa_sink_input_info_list_cb = pa_sink_input_info_cb_t(
                self.pa_sink_input_info_cb)
            self._pa_client_info_list_cb = pa_client_info_cb_t(
                self.pa_client_info_cb)
            self._pa_context_index_cb = pa_context_index_cb_t(
                self.pa_context_index_cb)

            self.pa_request_update()

            pa_context_set_subscribe_callback(
                self._context, self._pa_context_subscribe_cb, None);
            o = pa_context_subscribe(self._context, (pa_subscription_mask_t)
                                           (PA_SUBSCRIPTION_MASK_SINK|
                                            PA_SUBSCRIPTION_MASK_SINK_INPUT|
                                            PA_SUBSCRIPTION_MASK_CLIENT
                                            ), self._null_cb, None)
            pa_operation_unref(o)

        elif ctc == PA_CONTEXT_FAILED:
            pa_threaded_mainloop_signal(self.pa_mainloop, 0)
            sys.exit(1)

        elif ctc == PA_CONTEXT_TERMINATED:
            pa_threaded_mainloop_signal(self.pa_mainloop, 0)

    def pa_request_update(self):
        o = pa_context_get_client_info_list(
            self._context, self._pa_client_info_list_cb, None)
        pa_operation_unref(o)

        o = pa_context_get_sink_info_list(
            self._context, self._pa_sink_info_cb, None)
        pa_operation_unref(o)

        o = pa_context_get_sink_input_info_list(
            self._context, self._pa_sink_input_info_list_cb, True)
        pa_operation_unref(o)

    def pa_context_index_cb(self, context, index, user_data):
        return

    def pa_context_success_cb(self, context, c_int,  user_data):
        return

    def pa_context_subscribe_cb(self, context, event_type, index, user_data):
        et = event_type & PA_SUBSCRIPTION_EVENT_FACILITY_MASK

        if et == PA_SUBSCRIPTION_EVENT_CLIENT:

            if event_type & PA_SUBSCRIPTION_EVENT_TYPE_MASK \
              == PA_SUBSCRIPTION_EVENT_REMOVE:
                self.remove_client_cb(int(index))
            else:
                o = pa_context_get_client_info(
                    self._context, index, self._pa_client_info_list_cb,
                    None)
                pa_operation_unref(o)

        elif et == PA_SUBSCRIPTION_EVENT_SINK:
            if event_type & PA_SUBSCRIPTION_EVENT_TYPE_MASK \
              == PA_SUBSCRIPTION_EVENT_REMOVE:
                 self.remove_sink_cb(int(index))
            else:
                o = pa_context_get_sink_info_by_index(
                    self._context, int(index), self._pa_sink_info_cb, True)
                pa_operation_unref(o)

        elif et == PA_SUBSCRIPTION_EVENT_SINK_INPUT:
            if event_type & PA_SUBSCRIPTION_EVENT_TYPE_MASK \
              == PA_SUBSCRIPTION_EVENT_REMOVE:
                 self.remove_sink_input_cb(int(index))
            else:
                o = pa_context_get_sink_input_info(
                    self._context, int(index),
                     self._pa_sink_input_info_list_cb, True)
                pa_operation_unref(o)

    def dict_from_proplist(self, proplist):
        props = { }
        for prop in pa_proplist_to_string(proplist).split('\n'):
            left, _, right = prop.partition('=')
            props[left.strip()] = right.strip()[1:-1]
        return props

    def pa_client_info_cb(self, context, struct, c_int, user_data):
        if struct:
            self.new_client_cb(
                struct.contents.index, struct.contents,
                self.dict_from_proplist(struct.contents.proplist))

    def pa_sink_input_info_cb(self, context, struct, index, user_data):
        if struct and user_data:
            self.new_sink_input_cb(int(struct.contents.index), struct.contents)

    def pa_sink_info_cb(self, context, struct, index, data):
        if struct:
            self.new_sink_cb(
                int(struct.contents.index), struct.contents,
                self.dict_from_proplist(struct.contents.proplist))

    def set_sink_volume(self, index, cvolume):
        o = pa_context_set_sink_volume_by_index(
            self._context, index, cvolume, self._null_cb, None)
        pa_operation_unref(o)

    def set_sink_mute(self, index, mute):
        o = pa_context_set_sink_mute_by_index(
            self._context, index, mute, self._null_cb, None)
        pa_operation_unref(o)

    def set_sink_input_volume(self, index, cvolume):
        o = pa_context_set_sink_input_volume(
            self._context, index, cvolume, self._null_cb, None)
        pa_operation_unref(o)


class PulseAudioManager():
    def __init__(self, volctl):
        self.volctl = volctl
        self.pa_clients = {}
        self.pa_sinks = {}
        self.pa_sink_inputs = {}
        self.pa = PulseAudio(
            self.on_new_pa_client,
            self.on_remove_pa_client,
            self.on_new_pa_sink,
            self.on_remove_pa_sink,
            self.on_new_pa_sink_input,
            self.on_remove_pa_sink_input
        )

    def on_new_pa_client(self, index, struct, props):
        if not self.pa_clients.has_key(index):
            self.pa_clients[index] = Client(self, index, struct, props)
            self.pa_clients[index].update(struct, props)
        else:
            self.pa_clients[index].update(struct, props)

    def on_remove_pa_client(self, index):
        if self.pa_clients.has_key(index):
            client = self.pa_clients[index]
            del self.pa_clients[index]

    def on_new_pa_sink(self, index, struct, props):
        if not self.pa_sinks.has_key(index):
            self.pa_sinks[index] = Sink(self, index, struct, props)
            self.pa_sinks[index].update(struct, props)
            # notify gui thread
            gobject.idle_add(self.volctl.sink_count_changed)
        else:
            self.pa_sinks[index].update(struct, props)

    def on_remove_pa_sink(self, index):
        del self.pa_sinks[index]
        # notify gui thread
        gobject.idle_add(self.volctl.sink_count_changed)

    def on_new_pa_sink_input(self, index, struct):
        # filter out strange events
        if struct.name == 'audio-volume-change':
            return
        if not self.pa_sink_inputs.has_key(index):
            self.pa_sink_inputs[index] = SinkInput(self, index, struct)
            self.pa_sink_inputs[index].update(struct)
            # notify gui thread
            gobject.idle_add(self.volctl.sink_count_changed)
        else:
            self.pa_sink_inputs[index].update(struct)

    def on_remove_pa_sink_input(self, index):
        if self.pa_sink_inputs.has_key(index):
            del self.pa_sink_inputs[index]
            # notify gui thread
            gobject.idle_add(self.volctl.sink_count_changed)

    def _get_first_sink(self):
        try:
            return self.pa_sinks[self.pa_sinks.keys()[0]]
        except IndexError:
            return None

    # called Sink, SinkInput objects

    def set_sink_volume(self, index, cvolume):
        self.pa.set_sink_volume(index, cvolume)

    def set_sink_mute(self, index, mute):
        self.pa.set_sink_mute(index, mute)

    def set_sink_input_volume(self, index, cvolume):
        self.pa.set_sink_input_volume(index, cvolume)

    # called by gui thread -> lock pa thread

    def set_volume(self, volume):
        self._get_first_sink().set_volume(volume)

    def toggle_mute(self):
        sink = self._get_first_sink()
        sink.set_mute(not sink.mute)


class Sink():
    icon_name = 'audio-card'

    def __init__(self, pa_mgr, idx, struct, props):
        self.pa_mgr = pa_mgr
        self.idx = idx
        self.scale = None

    def update(self, struct, props):
        # set values
        self.name = struct.description
        self.volume = struct.volume.values[0]
        self.channels = struct.volume.channels
        self.mute = bool(struct.mute)
        # tray icon update (first sound card)
        if self == self.pa_mgr._get_first_sink():
            gobject.idle_add(
                self.pa_mgr.volctl.update_values, self.volume, self.mute)
        # scale update
        gobject.idle_add(self.pa_mgr.volctl.update_sink_scale, self.idx,
                         self.volume, self.mute)

    def set_volume(self, volume):
        self.volume = volume
        cvolume = cvolume_from_volume(volume, self.channels)
        self.pa_mgr.set_sink_volume(self.idx, cvolume)

    def set_mute(self, mute):
        self.mute = mute
        self.pa_mgr.set_sink_mute(self.idx, mute and 1 or 0)


class SinkInput():
    def __init__(self, pa_mgr, idx, struct):
        self.pa_mgr = pa_mgr
        self.idx = idx
        self.scale = None

    def update(self, struct):
        # set values
        self.volume = struct.volume.values[0]
        self.channels = struct.volume.channels
        self.mute = bool(struct.mute)
        self.client = struct.client
        # scale update
        gobject.idle_add(self.pa_mgr.volctl.update_sink_input_scale, self.idx,
                         self.volume, self.mute)

    def get_client(self):
        return self.pa_mgr.pa_clients[self.client]

    def set_volume(self, volume):
        self.volume = volume
        cvolume = cvolume_from_volume(volume, self.channels)
        self.pa_mgr.set_sink_input_volume(self.idx, cvolume)

    @property
    def icon_name(self):
        return self.get_client().icon_name

    @property
    def name(self):
        return self.get_client().name


class Client():
    def __init__(self, pa_mgr, idx, struct, props):
        self.pa_mgr = pa_mgr
        self.idx = idx

    def update(self, struct, props):
        self.name = struct.name
        try:
            self.icon_name = props['application.icon_name']
        except KeyError:
            try:
                self.icon_name = APP_TO_ICON_NAME[self.name]
            except KeyError:
                self.icon_name = 'multimedia-volume-control'
