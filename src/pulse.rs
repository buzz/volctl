use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use libpulse::callbacks::ListResult;
use libpulse::context::introspect::{ServerInfo, SinkInfo, SinkInputInfo};
use libpulse::context::subscribe::{Facility, InterestMaskSet, Operation};
use libpulse::context::{Context, FlagSet as CtxFlagSet, State as ContextState};
use libpulse::def::BufferAttr;
use libpulse::mainloop::threaded::Mainloop;
use libpulse::proplist::{Proplist, properties};
use libpulse::sample::{Format, Spec};
use libpulse::stream::{FlagSet as StreamFlagSet, PeekResult, Stream};
use libpulse::volume::{ChannelVolumes, Volume};

use crate::constants::{MAX_NATURAL_VOL, MAX_VOL_SCALE};
use crate::errors::PulseError;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum StreamType {
    #[default]
    Sink,
    SinkInput,
}

#[derive(Debug, Clone, Default)]
pub struct MeterData {
    pub t: StreamType,
    pub index: u32,

    pub name: String,
    pub icon: String,
    pub description: String,

    pub volume: ChannelVolumes,
    pub muted: bool,
}

/// The different message types that can be passed from the pulse thread to the data store.
enum TxMessage {
    DefaultSinkName(String),
    StreamUpdate(StreamType, Box<TxStreamData>),
    StreamRemove(StreamType, u32),
    Peak(StreamType, u32, u32),
}

/// Transferrable information pertaining to a stream.
#[derive(Debug)]
pub struct TxStreamData {
    pub data: MeterData,
    pub monitor_index: u32,
}

/// Stored representation of a pulse stream.
pub struct StreamData {
    pub data: MeterData,
    pub peak: u32,
    pub monitor_index: u32,
    /// Monitor stream connection. kept alive by this Rc.
    monitor: Rc<RefCell<Stream>>,
}

/// The main controller for all pulse server interactions.
pub struct Pulse {
    mainloop: Rc<RefCell<Mainloop>>,
    context: Rc<RefCell<Context>>,

    tx: Sender<TxMessage>,
    rx: Receiver<TxMessage>,

    pub default_sink: u32,
    pub active_sink: u32,

    sinks: HashMap<u32, StreamData>,
    sink_inputs: HashMap<u32, StreamData>,
}

impl Pulse {
    /// Creates a new pulse controller.
    pub fn new() -> Result<Self, PulseError> {
        let mut proplist = Proplist::new().ok_or(PulseError::ContextInit)?;
        proplist
            .set_str(properties::APPLICATION_NAME, "Myxer")
            .map_err(|_| PulseError::ContextInit)?;

        let mainloop = Rc::new(RefCell::new(
            Mainloop::new().ok_or(PulseError::MainloopInit)?,
        ));

        let context = Rc::new(RefCell::new(
            Context::new_with_proplist(&*mainloop.borrow(), "Myxer Context", &proplist)
                .ok_or(PulseError::ContextInit)?,
        ));

        let (tx, rx) = mpsc::channel();

        Ok(Pulse {
            mainloop,
            context,
            tx,
            rx,
            default_sink: u32::MAX,
            active_sink: u32::MAX,
            sinks: HashMap::new(),
            sink_inputs: HashMap::new(),
        })
    }

    /// Initiates a connection to pulse. Blocks until success.
    pub fn connect(&mut self) -> Result<(), PulseError> {
        // 1. Set up state callback
        {
            let mut ctx = self.context.borrow_mut();
            let ml_weak = Rc::downgrade(&self.mainloop);
            let ctx_weak = Rc::downgrade(&self.context);

            ctx.set_state_callback(Some(Box::new(move || {
                if let (Some(ml_rc), Some(ctx_rc)) = (ml_weak.upgrade(), ctx_weak.upgrade())
                    && let Ok(state) = ctx_rc.try_borrow().map(|c| c.get_state())
                    && matches!(
                        state,
                        ContextState::Ready | ContextState::Failed | ContextState::Terminated
                    )
                    && let Ok(mut ml) = ml_rc.try_borrow_mut()
                {
                    ml.signal(false);
                }
            })));

            ctx.connect(None, CtxFlagSet::NOFLAGS, None).map_err(|e| {
                PulseError::ConnectionFailed(
                    e.to_string().unwrap_or_else(|| "Unknown error".into()),
                )
            })?;
        }

        // 2. Start Mainloop
        {
            let mut ml = self.mainloop.borrow_mut();
            ml.lock();
            ml.start().map_err(|_| PulseError::MainloopStart)?;
            ml.unlock();
        }

        // 3. Wait for Ready state - Polling with Sleep
        // This avoids the ml.wait() deadlock entirely by not holding a lock
        // while the background thread is trying to signal or update state.
        loop {
            let state = {
                let mut ml = self.mainloop.borrow_mut();
                ml.lock();
                let s = self.context.borrow().get_state();
                ml.unlock();
                s
            };

            match state {
                ContextState::Ready => break,
                ContextState::Failed | ContextState::Terminated => {
                    self.mainloop.borrow_mut().stop();
                    return Err(PulseError::SessionTerminated);
                }
                _ => {
                    // Drop all borrows and sleep for a tiny bit.
                    // This gives the PulseAudio thread plenty of room to
                    // acquire the RefCell borrow and run callbacks.
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }

        // 4. Success
        self.context.borrow_mut().set_state_callback(None);
        self.subscribe();
        Ok(())
    }

    /// Sets the volume of the stream.
    pub fn set_volume(&self, t: StreamType, index: u32, volumes: ChannelVolumes) {
        let mut introspect = self.context.borrow().introspect();

        match t {
            StreamType::Sink => {
                let op = introspect.set_sink_volume_by_index(index, &volumes, None);
                // Prevent Operation::drop from calling pa_operation_unref,
                // which would destroy the operation before PulseAudio processes it.
                // This matches the Python pulsectl behavior (raw pointer is ignored).
                std::mem::forget(op);
            }
            StreamType::SinkInput => {
                let op = introspect.set_sink_input_volume(index, &volumes, None);
                std::mem::forget(op);
            }
        };
    }

    /// Mutes or unmutes a stream.
    pub fn set_muted(&self, t: StreamType, index: u32, mute: bool) {
        // Unmuting logic: restore volume if it was zeroed
        if !mute {
            let entry = match t {
                StreamType::Sink => self.sinks.get(&index),
                StreamType::SinkInput => self.sink_inputs.get(&index),
            };

            if let Some(entry) = entry {
                // If max volume is 0, reset to 100% (MAX_NATURAL_VOL)
                if entry.data.volume.max().0 == Volume::MUTED.0 {
                    let mut volumes = ChannelVolumes::default();
                    volumes.set_len(entry.data.volume.len());
                    volumes.set(entry.data.volume.len(), Volume(MAX_NATURAL_VOL));
                    self.set_volume(t, index, volumes);
                }
            }
        };

        let mut introspect = self.context.borrow().introspect();

        match t {
            StreamType::Sink => {
                let op = introspect.set_sink_mute_by_index(index, mute, None);
                std::mem::forget(op);
            }
            StreamType::SinkInput => {
                let op = introspect.set_sink_input_mute(index, mute, None);
                std::mem::forget(op);
            }
        };
    }

    /// Binds listeners to server events.
    fn subscribe(&mut self) {
        // Helper to send messages without panicking
        fn send_msg(tx: &Sender<TxMessage>, msg: TxMessage) {
            let _ = tx.send(msg); // Ignore error if receiver is dropped
        }

        // --- Callbacks ---
        // Note: These run in the PulseAudio thread. We must not panic here.

        fn tx_server(tx: &Sender<TxMessage>, item: &ServerInfo<'_>) {
            if let Some(name) = &item.default_sink_name {
                send_msg(tx, TxMessage::DefaultSinkName(name.clone().into_owned()));
            }
        }

        fn tx_sink(tx: &Sender<TxMessage>, result: ListResult<&SinkInfo<'_>>) {
            if let ListResult::Item(item) = result {
                let data = MeterData {
                    t: StreamType::Sink,
                    index: item.index,
                    icon: "audio-card".to_owned(),
                    name: item.name.clone().unwrap_or_default().into_owned(),
                    description: item.description.clone().unwrap_or_default().into_owned(),
                    volume: item.volume,
                    muted: item.mute,
                };

                send_msg(
                    tx,
                    TxMessage::StreamUpdate(
                        StreamType::Sink,
                        Box::new(TxStreamData {
                            data,
                            monitor_index: item.monitor_source,
                        }),
                    ),
                );
            }
        }

        fn tx_sink_input(tx: &Sender<TxMessage>, result: ListResult<&SinkInputInfo<'_>>) {
            if let ListResult::Item(item) = result {
                let icon = get_icon_name_from_sink_input(&item.proplist);
                let description = item
                    .proplist
                    .get_str("application.name")
                    .unwrap_or_default();

                let data = MeterData {
                    t: StreamType::SinkInput,
                    index: item.index,
                    icon,
                    name: item.name.clone().unwrap_or_default().into_owned(),
                    description,
                    volume: item.volume,
                    muted: item.mute,
                };

                send_msg(
                    tx,
                    TxMessage::StreamUpdate(
                        StreamType::SinkInput,
                        Box::new(TxStreamData {
                            data,
                            monitor_index: item.sink,
                        }),
                    ),
                );
            }
        }

        // Setup introspection and subscriptions
        let mut mainloop = self.mainloop.borrow_mut();
        mainloop.lock();

        let mut context = self.context.borrow_mut();
        let introspect = context.introspect();

        // Initial Data Fetch
        let tx = self.tx.clone();
        introspect.get_sink_info_list(move |res| tx_sink(&tx, res));
        let tx = self.tx.clone();
        introspect.get_sink_input_info_list(move |res| tx_sink_input(&tx, res));
        let tx = self.tx.clone();
        introspect.get_server_info(move |res| tx_server(&tx, res));

        // Event Subscriptions
        let tx = self.tx.clone();
        context.subscribe(
            InterestMaskSet::SERVER | InterestMaskSet::SINK | InterestMaskSet::SINK_INPUT,
            |_| (),
        );

        context.set_subscribe_callback(Some(Box::new(move |fac, op, index| {
            let tx = tx.clone();

            if let (Some(facility), Some(operation)) = (fac, op) {
                match facility {
                    Facility::Server => {
                        introspect.get_server_info(move |res| tx_server(&tx, res));
                    }
                    Facility::Sink => match operation {
                        Operation::Removed => {
                            send_msg(&tx, TxMessage::StreamRemove(StreamType::Sink, index))
                        }
                        _ => {
                            introspect.get_sink_info_by_index(index, move |res| tx_sink(&tx, res));
                        }
                    },
                    Facility::SinkInput => match operation {
                        Operation::Removed => {
                            send_msg(&tx, TxMessage::StreamRemove(StreamType::SinkInput, index))
                        }
                        _ => {
                            introspect
                                .get_sink_input_info(index, move |res| tx_sink_input(&tx, res));
                        }
                    },
                    _ => (),
                };
            }
        })));

        mainloop.unlock();
    }

    /// Handles queued messages from the pulse thread.
    /// Returns true if the layout needs a refresh.
    pub fn update(&mut self) -> bool {
        let mut received = false;

        // Drain the channel non-blocking
        while let Ok(msg) = self.rx.try_recv() {
            received = true;
            match msg {
                TxMessage::DefaultSinkName(sink) => self.update_default(sink),
                TxMessage::StreamUpdate(t, data) => {
                    // Log error but don't crash if stream creation fails
                    if let Err(e) = self.update_stream(t, &data) {
                        tracing::error!(error = %e, "Error updating stream");
                    }
                }
                TxMessage::StreamRemove(t, ind) => self.remove_stream(t, ind),
                TxMessage::Peak(t, ind, peak) => self.update_peak(t, ind, peak),
            }
        }

        received
    }

    // --- Accessors for immutable data ---

    pub fn get_sinks(&self) -> &HashMap<u32, StreamData> {
        &self.sinks
    }

    pub fn get_sink_inputs(&self) -> &HashMap<u32, StreamData> {
        &self.sink_inputs
    }

    // --- Private Internal Logic ---

    fn update_default(&mut self, sink_name: String) {
        for (i, v) in &self.sinks {
            if v.data.name == sink_name {
                self.default_sink = *i;
                self.active_sink = *i;
                break;
            }
        }
    }

    fn update_stream(
        &mut self,
        t: StreamType,
        stream_data: &TxStreamData,
    ) -> Result<(), PulseError> {
        let data = stream_data.data.clone();
        let index = data.index;

        let entry = match t {
            StreamType::Sink => self.sinks.get_mut(&index),
            StreamType::SinkInput => self.sink_inputs.get_mut(&index),
        };

        if let Some(stream) = entry {
            stream.data = data;
        } else {
            let source_str = stream_data.monitor_index.to_string();
            let monitor = self.create_monitor_stream(
                t,
                if t == StreamType::SinkInput {
                    None
                } else {
                    Some(&source_str)
                },
                index,
            )?;

            let stream_entry = StreamData {
                data,
                peak: 0,
                monitor_index: stream_data.monitor_index,
                monitor,
            };

            match t {
                StreamType::Sink => self.sinks.insert(index, stream_entry),
                StreamType::SinkInput => self.sink_inputs.insert(index, stream_entry),
            };
        }
        Ok(())
    }

    fn remove_stream(&mut self, t: StreamType, index: u32) {
        let stream_opt = match t {
            StreamType::Sink => self.sinks.get_mut(&index),
            StreamType::SinkInput => self.sink_inputs.get_mut(&index),
        };

        if let Some(stream) = stream_opt {
            let mut monitor = stream.monitor.borrow_mut();
            let mut mainloop = self.mainloop.borrow_mut();
            mainloop.lock();

            if monitor.get_state().is_good() {
                monitor.set_read_callback(None);
                let _ = monitor.disconnect();
            }

            mainloop.unlock();
        }

        match t {
            StreamType::Sink => self.sinks.remove(&index),
            StreamType::SinkInput => self.sink_inputs.remove(&index),
        };
    }

    fn update_peak(&mut self, t: StreamType, index: u32, peak: u32) {
        match t {
            StreamType::Sink => {
                if let Some(e) = self.sinks.get_mut(&index) {
                    e.peak = peak;
                }
            }
            StreamType::SinkInput => {
                if let Some(e) = self.sink_inputs.get_mut(&index) {
                    e.peak = peak;
                }
            }
        };
    }

    fn create_monitor_stream(
        &mut self,
        t: StreamType,
        source: Option<&str>,
        stream_index: u32,
    ) -> Result<Rc<RefCell<Stream>>, PulseError> {
        let attr = BufferAttr {
            fragsize: 4,
            maxlength: u32::MAX,
            ..Default::default()
        };

        let spec = Spec {
            channels: 1,
            format: Format::F32le,
            rate: 30, // Low rate for UI updates
        };

        if !spec.is_valid() {
            return Err(PulseError::StreamCreation);
        }

        let stream = {
            let mut ctx = self.context.borrow_mut();
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

            let mut mainloop = self.mainloop.borrow_mut();
            mainloop.lock();

            let res = stream_mut.connect_record(
                source,
                Some(&attr),
                StreamFlagSet::DONT_MOVE
                    | StreamFlagSet::ADJUST_LATENCY
                    | StreamFlagSet::PEAK_DETECT,
            );

            mainloop.unlock();

            if res.is_err() {
                return Err(PulseError::StreamCreation);
            }

            // Setup read callback
            let stream_clone = stream.clone();
            let tx = self.tx.clone();

            stream_mut.set_read_callback(Some(Box::new(move |_| {
                // IMPORTANT: We are in the pulse thread.
                // Borrowing stream_clone is safe because set_read_callback implies strict ownership rules
                // and the main loop is locked during this callback.
                monitor_read_callback(&mut stream_clone.borrow_mut(), t, stream_index, &tx);
            })));
        }

        Ok(stream)
    }
}

/// Resolve the icon name for a sink input, following the same fallback chain as the Python
/// implementation (`slider_win._icon_name_from_sink_input`).
///
/// Fallback order:
/// 1. `application.icon_name` property
/// 2. `media.icon_name` property
/// 3. `application.process.binary` property (e.g., "firefox")
/// 4. `application.name` lowercased with spaces replaced by dashes
/// 5. If no icon name found at all → `"multimedia-volume-control"`
fn get_icon_name_from_sink_input(proplist: &libpulse::proplist::Proplist) -> String {
    proplist
        .get_str("application.icon_name")
        .or_else(|| proplist.get_str("media.icon_name"))
        .or_else(|| proplist.get_str("application.process.binary"))
        .or_else(|| {
            proplist
                .get_str("application.name")
                .map(|name| name.to_lowercase().replace(' ', "-"))
        })
        .map(|s| s.to_owned())
        .unwrap_or_else(|| "multimedia-volume-control".to_owned())
}

/// Helper function to process raw audio bytes into a peak value.
/// Isolates the specific math scaling logic.
fn calculate_peak(raw_peak: f32) -> u32 {
    (raw_peak.sqrt() * MAX_NATURAL_VOL as f32 * MAX_VOL_SCALE as f32).round() as u32
}

/// Standalone callback logic for monitors.
fn monitor_read_callback(stream: &mut Stream, t: StreamType, index: u32, tx: &Sender<TxMessage>) {
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
            _ => break,
        }
    }

    if raw_peak > 0.0 {
        let peak = calculate_peak(raw_peak);
        let _ = tx.send(TxMessage::Peak(t, index, peak));
    }
}

// Clean up resources when Pulse is dropped
impl Drop for Pulse {
    fn drop(&mut self) {
        // Disconnect streams
        for stream in self.sinks.values() {
            let _ = stream.monitor.borrow_mut().disconnect();
        }
        for stream in self.sink_inputs.values() {
            let _ = stream.monitor.borrow_mut().disconnect();
        }

        // Stop mainloop
        if let Ok(mut ml) = self.mainloop.try_borrow_mut() {
            ml.stop();
        }
    }
}
