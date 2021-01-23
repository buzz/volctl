from contextlib import contextmanager
import threading

from gi.repository import GLib
from pulsectl import Pulse, PulseDisconnected, PulseEventMaskEnum, PulseEventTypeEnum
from pulsectl.pulsectl import c

from volctl.meta import PROGRAM_NAME


def get_by_attr(list_, attr_name, val):
    """Find by attribute in list of objects."""
    return next((s for s in list_ if getattr(s, attr_name) == val), None)


class PulsePoller(threading.Thread):
    """TODO"""

    def __init__(self, pulse, pulse_lock, pulse_hold, handle_event):
        super().__init__(name="volctl-pulsepoller", daemon=True)
        self.quit = False
        self._pulse_lock, self._pulse_hold = pulse_lock, pulse_hold
        self._pulse = pulse
        self._handle_event = handle_event

    def run(self):
        self._pulse.event_mask_set(
            # Use generated-members once in pylint:
            # https://github.com/graingert/pylint/commit/9bd38da10e2aca9a468d26774bb3283e2f2b30c6
            # pylint: disable=no-member
            PulseEventMaskEnum.sink,
            PulseEventMaskEnum.sink_input,
        )
        self._pulse.event_callback_set(self._callback)
        # delay_iter = cb_delay_iter(ev_cb, self.conf.force_refresh_interval)
        while True:
            with self._pulse_hold:
                self._pulse_lock.acquire()
            try:
                self._pulse.event_listen(0.2)
                # self.pulse.event_listen(next(delay_iter))
                if self.quit:
                    break
            except PulseDisconnected:
                print("pulsectl disconnected")
                # wakeup_handler(disconnected=True)
                break
            finally:
                self._pulse_lock.release()

    def _callback(self, event):
        if self.is_alive():
            GLib.idle_add(self._handle_event, event)


class PulseManager:
    """Manage connection to PulseAudio and receive updates."""

    def __init__(self, volctl):
        self._volctl = volctl
        self._pulse = Pulse(PROGRAM_NAME)
        self._event_listen_stopped = False

        # Start polling thread
        self._pulse_lock, self._pulse_hold = threading.Lock(), threading.Lock()
        self._poller_thread = PulsePoller(
            self._pulse, self._pulse_lock, self._pulse_hold, self._handle_event
        )
        self._poller_thread.start()

        # Stream monitoring
        self._monitor_streams = {}
        self._read_cb_ctypes = c.PA_STREAM_REQUEST_CB_T(self._read_cb)
        self._samplespec = c.PA_SAMPLE_SPEC(
            format=c.PA_SAMPLE_FLOAT32BE, rate=25, channels=1
        )

    def close(self):
        """Close the PulseAudio connection and event polling thread."""
        self.stop_peak_monitor()
        if self._poller_thread and self._poller_thread.is_alive():
            self._poller_thread.quit = True
            self._poller_thread.join(timeout=1.0)
        if self._pulse:
            self._pulse.close()

    @contextmanager
    # def update_wakeup(self, trap_errors=True, loop_interval=0.03):
    def update_wakeup(self, loop_interval=0.03):
        "Anything pulse-related MUST be done in this context."
        if self._event_listen_stopped:
            yield self._pulse
        else:
            # Stop PA event listen loop, so we can call PA functions.
            with self._pulse_hold:
                for _ in range(int(2.0 / loop_interval)):
                    # wakeup only works when loop is actually started,
                    #  which might not be the case regardless of any locks.
                    self._pulse.event_listen_stop()
                    if self._pulse_lock.acquire(timeout=loop_interval):
                        break
                else:
                    raise RuntimeError("poll_wakeup() hangs, likely locking issue")
                try:
                    self._event_listen_stopped = True
                    yield self._pulse
                # except Exception as err:
                #     if not trap_errors:
                #         self._update_wakeup_break = True
                #         raise
                #     print(
                #         "Pulse interaction failure, skipping: "
                #         f"<{err.__class__.__name__}> {err}"
                #     )
                finally:
                    self._pulse_lock.release()
                    self._event_listen_stopped = False

    def _handle_event(self, event):
        """Handle PulseAudio event."""
        fac = "sink" if event.facility == "sink" else "sink_input"

        if event.t == PulseEventTypeEnum.change:  # pylint: disable=no-member
            method, obj = None, None

            with self.update_wakeup() as pulse:
                obj_list = getattr(pulse, f"{fac}_list")()
                obj = get_by_attr(obj_list, "index", event.index)

            if obj:
                method = getattr(self._volctl, f"{fac}_update")
                method(event.index, obj.volume.value_flat, obj.mute == 1)

        elif event.t in (
            # pylint: disable=no-member
            PulseEventTypeEnum.new,
            PulseEventTypeEnum.remove,
        ):
            self._volctl.slider_count_changed()

        else:
            print(f"Warning: Unhandled event type for {fac}: {event.t}")

    def _read_cb(self, stream, nbytes, idx):
        data = c.c_void_p()
        nbytes = c.c_int(nbytes)
        c.pa.stream_peek(stream, data, c.byref(nbytes))
        try:
            if not data or nbytes.value < 1:
                return
            samples = c.cast(data, c.POINTER(c.c_float))
            val = max(samples[i] for i in range(nbytes.value))
        finally:
            # stream_drop() flushes buffered data (incl. buff=NULL "hole" data)
            # stream.h: "should not be called if the buffer is empty"
            if nbytes:
                c.pa.stream_drop(stream)
        GLib.idle_add(self._volctl.peak_update, idx, min(val, 1.0))

    def start_peak_monitor(self):
        """Start peak monitoring for all sinks and sink inputs."""
        with self.update_wakeup() as pulse:
            for sink in pulse.sink_list():
                stream = self._create_peak_stream(sink.index)
                self._monitor_streams[sink.index] = stream
            for sink_input in pulse.sink_input_list():
                sink_idx = self._pulse.sink_input_info(sink_input.index).sink
                stream = self._create_peak_stream(sink_idx, sink_input.index)
                self._monitor_streams[sink_input.index] = stream

    def stop_peak_monitor(self):
        """Stop peak monitoring for all sinks and sink inputs."""
        with self.update_wakeup():
            for idx, stream in self._monitor_streams.items():
                try:
                    c.pa.stream_disconnect(stream)
                except c.pa.CallError:
                    pass  # stream was removed
                finally:
                    GLib.idle_add(self._volctl.peak_update, idx, 0.0)
        self._monitor_streams = {}

    def _create_peak_stream(self, sink_idx, sink_input_idx=None):
        # Cannot use `get_peak_sample` from python-pulse-control as it would block GUI.
        proplist = c.pa.proplist_from_string(  # Hide this stream in mixer apps
            "application.id=org.PulseAudio.pavucontrol"
        )
        pa_context = self._pulse._ctx  # pylint: disable=protected-access
        idx = sink_idx if sink_input_idx is None else sink_input_idx
        stream = c.pa.stream_new_with_proplist(
            pa_context, f"peak {idx}", c.byref(self._samplespec), None, proplist
        )
        c.pa.proplist_free(proplist)
        c.pa.stream_set_read_callback(stream, self._read_cb_ctypes, idx)
        if sink_input_idx is not None:
            # Monitor single sink input
            c.pa.stream_set_monitor_stream(stream, sink_input_idx)
        c.pa.stream_connect_record(
            stream,
            str(sink_idx).encode("utf-8"),
            c.PA_BUFFER_ATTR(fragsize=4, maxlength=2 ** 32 - 1),
            c.PA_STREAM_DONT_MOVE
            | c.PA_STREAM_PEAK_DETECT
            | c.PA_STREAM_ADJUST_LATENCY
            | c.PA_STREAM_DONT_INHIBIT_AUTO_SUSPEND,
        )
        return stream

    # Set/get PulseAudio entities

    def set_main_volume(self, val):
        """Set default sink volume."""
        with self.update_wakeup() as pulse:
            pulse.volume_set_all_chans(self.default_sink, val)

    def toggle_main_mute(self):
        """Toggle default sink mute."""
        with self.update_wakeup():
            self.sink_set_mute(self.default_sink_idx, not self.default_sink.mute)

    def sink_set_mute(self, idx, mute):
        """Set sink mute."""
        with self.update_wakeup() as pulse:
            pulse.sink_mute(idx, mute)

    def sink_input_set_mute(self, idx, mute):
        """Set sink input mute."""
        with self.update_wakeup() as pulse:
            pulse.sink_input_mute(idx, mute)

    @property
    def volume(self):
        """Volume of the default sink."""
        return self.default_sink.volume.value_flat

    @property
    def mute(self):
        """Mute state of the default sink."""
        return self.default_sink.mute == 1

    @property
    def default_sink(self):
        """Default sink."""
        with self.update_wakeup() as pulse:
            sink_name = pulse.server_info().default_sink_name
            return get_by_attr(pulse.sink_list(), "name", sink_name)

    @property
    def default_sink_idx(self):
        """Default sink index."""
        sink = self.default_sink
        if sink:
            return sink.index
        return None
