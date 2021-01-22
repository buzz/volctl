from contextlib import contextmanager
import threading

from gi.repository import GLib
from pulsectl import Pulse, PulseDisconnected, PulseEventMaskEnum, PulseEventTypeEnum

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

    def close(self):
        """Close the PulseAudio connection and event polling thread."""
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
