use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use libpulse::context::Context;
use libpulse::def::BufferAttr;
use libpulse::mainloop::threaded::Mainloop;
use libpulse::sample::{Format, Spec};
use libpulse::stream::{FlagSet as StreamFlagSet, PeekResult, State as StreamState, Stream};

use super::types::{StreamType, calculate_peak};
use crate::constants::PEAKS_RATE;
use crate::errors::PulseError;

/// Message types sent from the PulseAudio thread to the main thread.
pub enum TxMessage {
    DefaultSinkName(String),
    StreamUpdate(StreamType, Box<super::types::TxStreamData>),
    StreamRemove(StreamType, u32),
    Peak(StreamType, u32, u32),
    /// Sent when a monitor stream is suspended (e.g., the audio app paused/stopped).
    /// Triggers immediate peak decay to zero for the affected stream.
    PeakZero(StreamType, u32),
}

/// Create a monitor stream for peak detection on a given sink or sink input.
pub fn create_monitor_stream(
    context: &RefCell<Context>,
    mainloop: &RefCell<Mainloop>,
    t: StreamType,
    source: Option<&str>,
    stream_index: u32,
    tx: &Sender<TxMessage>,
    vu_enabled: &Arc<AtomicBool>,
) -> Result<Rc<RefCell<Stream>>, PulseError> {
    let attr = BufferAttr {
        fragsize: 4,
        maxlength: u32::MAX,
        ..Default::default()
    };

    let spec = Spec {
        channels: 1,
        format: Format::F32le,
        rate: PEAKS_RATE,
    };

    if !spec.is_valid() {
        return Err(PulseError::StreamCreation);
    }

    let stream = {
        let mut ctx = context.borrow_mut();
        match Stream::new(&mut ctx, "Peak Detect", &spec, None) {
            Some(s) => Rc::new(RefCell::new(s)),
            None => return Err(PulseError::StreamCreation),
        }
    };

    {
        let mut stream_mut = stream.borrow_mut();
        if t == StreamType::SinkInput && stream_mut.set_monitor_stream(stream_index).is_err() {
            return Err(PulseError::StreamCreation);
        }

        // Setup read callback BEFORE connect_record to avoid a race where
        // PulseAudio delivers data before the handler is installed.
        let stream_clone = stream.clone();
        let tx_read = tx.clone();
        let tx_suspend = tx.clone();
        let vu_enabled = vu_enabled.clone();

        stream_mut.set_read_callback(Some(Box::new(move |_| {
            // IMPORTANT: We are in the pulse thread.
            // Borrowing stream_clone is safe because set_read_callback implies strict ownership rules
            // and the main loop is locked during this callback.
            monitor_read_callback(
                &mut stream_clone.borrow_mut(),
                t,
                stream_index,
                &tx_read,
                &vu_enabled,
            );
        })));

        // Setup suspended callback: when the stream is suspended (e.g., the
        // monitored audio app pauses/stops), immediately zero the peak.
        // This matches pavucontrol's behavior of calling decayToZero().
        stream_mut.set_suspended_callback(Some(Box::new(move || {
            let _ = tx_suspend.send(TxMessage::PeakZero(t, stream_index));
        })));

        let mut mainloop = mainloop.borrow_mut();
        mainloop.lock();

        // Build flags: always use DONT_MOVE, ADJUST_LATENCY, PEAK_DETECT.
        // For sink inputs, add DONT_INHIBIT_AUTO_SUSPEND so PA can auto-suspend
        // the monitor stream when the app is idle (saves CPU).
        //
        // NOTE: We don't use START_CORKED here. When VU is disabled, we skip
        // setting the read callback, which already prevents any peak processing.
        // Using START_CORKED would require tracking uncork timing vs stream READY
        // state transitions, which is fragile.
        let flags = StreamFlagSet::DONT_MOVE
            | StreamFlagSet::ADJUST_LATENCY
            | StreamFlagSet::PEAK_DETECT
            | if t == StreamType::SinkInput {
                StreamFlagSet::DONT_INHIBIT_AUTO_SUSPEND
            } else {
                StreamFlagSet::empty()
            };

        let res = stream_mut.connect_record(source, Some(&attr), flags);

        mainloop.unlock();

        if res.is_err() {
            return Err(PulseError::StreamCreation);
        }
    }

    Ok(stream)
}

/// Standalone callback logic for monitors.
/// Always reads and discards audio data (prevents buffer overflow),
/// but only sends peak messages when `vu_enabled` is true.
fn monitor_read_callback(
    stream: &mut Stream,
    t: StreamType,
    index: u32,
    tx: &Sender<TxMessage>,
    vu_enabled: &Arc<AtomicBool>,
) {
    let mut raw_peak: f32 = 0.0;

    while stream.readable_size().unwrap_or(0) > 0 {
        match stream.peek() {
            Ok(PeekResult::Data(bytes)) => {
                // Convert slice to array safely
                if let Ok(buf) = bytes.try_into() {
                    let val = f32::from_le_bytes(buf);
                    raw_peak = val.max(raw_peak);
                }
                let _ = stream.discard();
            }
            Ok(PeekResult::Hole(_)) => {
                let _ = stream.discard();
            }
            Ok(PeekResult::Empty) => break,
            Err(e) => {
                tracing::warn!(error = %e, "peek() failed on monitor stream, stopping read loop");
                break;
            }
        }
    }

    // Only send peak messages when VU monitoring is active
    if raw_peak > 0.0 && vu_enabled.load(Ordering::Relaxed) {
        let peak = calculate_peak(raw_peak);
        let _ = tx.send(TxMessage::Peak(t, index, peak));
    }
}

/// Enable or disable VU peak monitoring on all monitor streams.
/// When disabled, streams are corked (stops data flow) but the read
/// callback stays installed. Removing and re-adding the callback breaks
/// PulseAudio's PEAK_DETECT state, causing peaks to stop updating.
pub fn set_vu_enabled(
    mainloop: &RefCell<Mainloop>,
    streams: &mut [Rc<RefCell<Stream>>],
    enabled: bool,
) {
    let mut mainloop = mainloop.borrow_mut();
    mainloop.lock();

    for stream_rc in streams {
        let mut monitor = stream_rc.borrow_mut();
        // Only operate on streams that are fully connected (READY).
        // Streams in CREATING state will be handled when they transition.
        if monitor.get_state() == StreamState::Ready {
            if enabled {
                // Uncork to resume data flow. The read callback is already
                // installed from stream creation and stays active.
                if monitor.is_corked().unwrap_or(false) {
                    let _ = monitor.uncork(Some(Box::new(move |_| {})));
                }
            } else {
                // Cork to stop data flow. The callback stays installed
                // so PEAK_DETECT state is preserved.
                if !monitor.is_corked().unwrap_or(true) {
                    let _ = monitor.cork(Some(Box::new(move |_| {})));
                }
            }
        }
    }

    mainloop.unlock();
}
