// Copyright (c) 2024 Mike Tsao

//! The traits that define many characteristics and relationships among parts of
//! the system.

use crate::{
    prelude::*,
    types::{MidiEvent, MidiNote},
};
use crossbeam::channel::{Receiver, Sender};
#[cfg(feature = "egui")]
use strum_macros::Display;

/// Quick import of all important traits.
pub mod prelude {
    pub use super::{
        CanPrototype, Configurable, Configurables, ControlEventsFn, ControlProxyEventsFn,
        Controllable, Controls, ControlsAsProxy, Entity, Generates, GeneratesEnvelope,
        GenerationBuffer, HandlesMidi, HasExtent, HasMetadata, HasSettings, IsStereoSampleVoice,
        IsVoice, MidiMessagesFn, MidiNoteLabelMetadata, PlaysNotes, ProvidesService, Sequences,
        SequencesMidi, Serializable, StoresVoices, TransformsAudio, WorkEvent,
    };
    #[cfg(feature = "egui")]
    pub use super::{Displays, DisplaysAction};
}

// We re-export here so that consumers of traits don't have to worry as much
// about exactly where they are in the code, but those working on the code can
// still organize them.
pub use crate::automation::{
    ControlEventsFn, ControlProxyEventsFn, Controllable, Controls, ControlsAsProxy,
};

/// A convenience struct for the fields implied by [Configurable]. Note that
/// this struct is not serde-compliant, because these fields typically aren't
/// meant to be serialized.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Configurables {
    sample_rate: SampleRate,
    tempo: Tempo,
    time_signature: TimeSignature,
}
impl Configurable for Configurables {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate
    }

    fn tempo(&self) -> Tempo {
        self.tempo
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.tempo = tempo
    }

    fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature
    }
}

/// Something that is [Configurable] is interested in staying in sync with
/// global configuration.
pub trait Configurable {
    /// Returns this item's sample rate.
    fn sample_rate(&self) -> SampleRate {
        // I was too lazy to add this everywhere when I added this to the trait,
        // but I didn't want unexpected usage to go undetected.
        unimplemented!("Someone asked for a SampleRate but we provided default");
    }

    /// The sample rate changed.
    #[allow(unused_variables)]
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {}

    /// Returns this item's [Tempo].
    fn tempo(&self) -> Tempo {
        unimplemented!("Someone forgot to implement tempo()")
    }

    /// Tempo (beats per minute) changed.
    #[allow(unused_variables)]
    fn update_tempo(&mut self, tempo: Tempo) {}

    /// Returns this item's [TimeSignature].
    fn time_signature(&self) -> TimeSignature {
        unimplemented!("Someone forgot to implement time_signature()")
    }

    /// The global time signature changed. Recipients are free to ignore this if
    /// they are dancing to their own rhythm (e.g., a polyrhythmic pattern), but
    /// they still want to know it, because they might perform local Time
    /// Signature L in terms of global Time Signature G.
    #[allow(unused_variables)]
    fn update_time_signature(&mut self, time_signature: TimeSignature) {}

    /// Sent to indicate that it's time to reset internal state. Oscillators
    /// should reset phase, etc.
    fn reset(&mut self) {}
}

/// Describes the public interface of an envelope generator, which provides a
/// normalized amplitude (0.0..=1.0) that changes over time according to its
/// internal parameters, external triggers, and the progression of time.
pub trait GeneratesEnvelope: Generates<Normal> + Send + core::fmt::Debug {
    /// Triggers the envelope's active stage.
    fn trigger_attack(&mut self);

    /// Triggers the end of the envelope's active stage.
    fn trigger_release(&mut self);

    /// Requests a fast decrease to zero amplitude. Upon reaching zero, switches
    /// to idle. If the EG is already idle, then does nothing. For normal EGs,
    /// the EG's settings (ADSR, etc.) don't affect the rate of shutdown decay.
    ///
    /// See DSSPC, 4.5 Voice Stealing, for an understanding of how the shutdown
    /// state helps. TL;DR: if we have to steal one voice to play a different
    /// note, it sounds better if the voice very briefly stops and restarts.
    fn trigger_shutdown(&mut self);

    /// Whether the envelope generator is in the idle state, which usually means
    /// quiescent and zero amplitude.
    fn is_idle(&self) -> bool;
}

/// A [PlaysNotes] turns note events into sound. It seems to overlap with
/// [HandlesMidi]; the reason it exists is to allow the two interfaces to evolve
/// independently, because MIDI is unlikely to be perfect for all our needs.
pub trait PlaysNotes {
    /// Whether the entity is currently making sound.
    fn is_playing(&self) -> bool;

    /// Initiates a note-on event. Depending on implementation, might initiate a
    /// steal (tell envelope to go to shutdown state, then do note-on when
    /// that's done).
    fn note_on(&mut self, key: u7, velocity: u7);

    /// Initiates an aftertouch event.
    fn aftertouch(&mut self, velocity: u7);

    /// Initiates a note-off event, which can take a long time to complete,
    /// depending on how long the envelope's release is.
    fn note_off(&mut self, velocity: u7);
}

/// A [StoresVoices] provides access to a collection of voices for a polyphonic
/// synthesizer. Different implementers provide different policies for how to
/// handle voice-stealing.
pub trait StoresVoices: Generates<StereoSample> + Send + Sync + core::fmt::Debug {
    /// The associated type of sample generator for this voice store.
    type Voice;

    /// Generally, this value won't change after initialization, because we try
    /// not to dynamically allocate new voices.
    fn voice_count(&self) -> usize;

    /// The number of voices reporting is_playing() true.
    fn active_voice_count(&self) -> usize;

    /// Fails if we run out of idle voices and can't steal any active ones.
    fn get_voice(&mut self, key: &u7) -> anyhow::Result<&mut Box<Self::Voice>>;

    /// All the voices.
    // Thanks to https://stackoverflow.com/a/58612273/344467 for the lifetime
    // magic
    fn voices<'a>(&'a self) -> Box<dyn Iterator<Item = &Box<Self::Voice>> + 'a>;

    /// All the voices as a mutable iterator.
    fn voices_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Box<Self::Voice>> + 'a>;
}

/// A synthesizer is composed of Voices. Ideally, a synth will know how to
/// construct Voices, and then handle all the MIDI events properly for them.
pub trait IsVoice<V: Default + Clone>: Generates<V> + PlaysNotes + Send + Sync {}
/// Same as IsVoice, but stereo.
pub trait IsStereoSampleVoice: IsVoice<StereoSample> {}

/// A convenience trait that helps describe the lifetime, in MusicalTime, of
/// something.
///
/// This is not necessarily the times of the first and last MIDI events. For
/// example, if the struct in question (MU, or Musical Unit) were one-measure
/// patterns, then the extent of such a pattern would be the full measure, even
/// if the pattern were empty, because it still takes up a measure of "musical
/// space."
///
/// Note that extent() returns a Range, not a RangeInclusive. This is most
/// natural for MUs like patterns that are aligned to musical boundaries. For a
/// MU that is instantaneous, like a MIDI event, however, the current
/// recommendation is to return a range whose end is the last event's time + one
/// MusicalTime unit, which adheres to the contract of Range, but can add an
/// extra measure of silence (since the range now extends to the next measure)
/// if the consumer of extent() doesn't understand what it's looking at.
pub trait HasExtent {
    /// Returns the range of MusicalTime that this thing spans.
    fn extent(&self) -> TimeRange;

    /// Sets the range.
    fn set_extent(&mut self, extent: TimeRange);

    /// Convenience method that returns the distance between extent's start and
    /// end. The duration is the amount of time from the start to the point when
    /// the next contiguous musical item should start. This does not necessarily
    /// mean the time between the first note-on and the first note-off! For
    /// example, an empty 4/4 pattern lasts for 4 beats.
    fn duration(&self) -> MusicalTime {
        let e = self.extent();
        e.0.end - e.0.start
    }
}

/// Implementers of [Controls] produce these events. Only the system receives
/// them; rather than forwarding them directly, the system converts them into
/// something else that might then get forwarded to recipients.
#[derive(Clone, Debug, PartialEq)]
pub enum WorkEvent {
    /// A MIDI message sent to a channel.
    Midi(MidiChannel, MidiMessage),

    /// A MIDI message that's limited to a specific track. Lower-level
    /// [WorkEvent::Midi] messages are decorated with the track information when
    /// passing to higher-level processors.
    MidiForTrack(TrackUid, MidiChannel, MidiMessage),

    /// A control event. Indicates that the sender's value has changed, and that
    /// subscribers should receive the update. This is how we perform
    /// automation: a controller produces a [WorkEvent::Control] message, and
    /// the system transforms it into [Controllable::control_set_param_by_index]
    /// method calls to inform subscribing entities that their linked parameters
    /// should change.
    Control(ControlValue),
}

/// Something that is [Serializable] might need to do work right before
/// serialization, or right after deserialization. These are the hooks.
pub trait Serializable {
    /// Called just before saving to disk.
    fn before_ser(&mut self) {}
    /// Called just after loading from disk.
    fn after_deser(&mut self) {}
}

/// Service methods.
///
/// A service is something that usually runs in its own thread as a daemon and
/// that communicates with clients by crossbeam channels. It accepts Inputs and
/// produces Events.
pub trait ProvidesService<I: core::fmt::Debug, E: core::fmt::Debug> {
    /// The sender side of the Input channel. Use this to send commands to the
    /// service.
    fn sender(&self) -> &Sender<I>;

    /// A convenience method to send Inputs to the service. Calling this implies
    /// that the caller has kept a reference to the service, which is uncommon,
    /// as the main value of services is to be able to clone senders with
    /// reckless abandon.
    fn send_input(&self, input: I) {
        if let Err(e) = self.sender().try_send(input) {
            eprintln!("While sending: {e:?}");
        }
    }

    /// The receiver side of the Event channel. Integrate this into a listener
    /// loop to respond to events.
    fn receiver(&self) -> &Receiver<E>;

    /// A convenience method to receive either Inputs or Events inside a
    /// crossbeam select loop. Unlike send_input(), this one is used frequently
    /// because it doesn't require use of &self.
    fn recv_operation<T>(
        oper: crossbeam::channel::SelectedOperation,
        r: &Receiver<T>,
    ) -> Result<T, crossbeam::channel::RecvError> {
        let input_result = oper.recv(r);
        if let Err(e) = input_result {
            eprintln!(
                "ProvidesService: While attempting to receive from {:?}: {}",
                *r, e
            );
        }
        input_result
    }
}

/// A [HasMetadata] has basic information about an [Entity]. Some methods apply
/// to the "class" of [Entity] (for example, all `ToyInstrument`s share the name
/// "ToyInstrument"), and others apply to each instance of a class (for example,
/// one ToyInstrument instance might be Uid 42, and another Uid 43).
pub trait HasMetadata {
    /// The [Uid] is a globally unique identifier for an instance of an
    /// [Entity].
    fn uid(&self) -> Uid;
    /// Assigns a [Uid].
    fn set_uid(&mut self, uid: Uid);
    /// A string that describes this class of [Entity]. Suitable for debugging
    /// or quick-and-dirty UIs.
    fn name(&self) -> &'static str;
    /// A kebab-case string that identifies this class of [Entity].
    fn key(&self) -> &'static str;
}

/// Something that [CanPrototype] can make another of its kind, but it's a
/// little smarter than [Clone]. Not every one of its fields should be cloned --
/// for example, a cache -- and this trait's methods know which is which.
///
/// TODO: this trait overlaps with Serde's functionality. Most fields that are
/// #[serde(skip)] would also be excluded here. Is there a way to hook into
/// Serde and derive the make_another() functionality from it?
pub trait CanPrototype: core::fmt::Debug + Default {
    /// Treats self as a prototype and makes another.
    fn make_another(&self) -> Self {
        let mut r = Self::default();
        r.update_from_prototype(self);
        r
    }

    /// Given another of this kind, updates its fields using self as a
    /// prototype.
    fn update_from_prototype(&mut self, prototype: &Self) -> &Self;
}

/// The actions that might result from [Displays::ui()].
#[cfg(feature = "egui")]
#[derive(Debug, Display)]
pub enum DisplaysAction {
    /// During the ui() call, the entity determined that something wants to link
    /// with us at control param index ControlIndex.
    Link(ControlLinkSource, ControlIndex),
}

#[cfg(feature = "egui")]
/// Something that can be called during egui rendering to display a view of
/// itself.
//
// Adapted from egui_demo_lib/src/demo/mod.rs
pub trait Displays {
    /// Renders this Entity. Returns a [Response](eframe::egui::Response).
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }

    /// Sets the [DisplaysAction] that resulted from this layout.
    #[allow(unused_variables)]
    fn set_action(&mut self, action: DisplaysAction) {}
    /// Also resets the action to None
    fn take_action(&mut self) -> Option<DisplaysAction> {
        None
    }

    /// Indicates which section of the timeline is being displayed. Entities
    /// that don't render in the timeline can ignore this.
    #[allow(unused_variables)]
    fn set_view_range(&mut self, view_range: &ViewRange) {}
}
/// Disabled (requires feature `egui`)
#[cfg(not(feature = "egui"))]
pub trait Displays {}

/// If an instrument responds to only a subset of possible MIDI notes, then it
/// can describe them here. Drumkits will typically override this method to
/// provide sample names for each note (Kick 1, Snare 3, etc).
#[derive(Debug)]
pub struct MidiNoteLabelMetadata {
    /// The contiguous range of recognized [MidiNote]s.
    pub range: core::ops::RangeInclusive<MidiNote>,
    /// One label for each [MidiNote] in the range.
    pub labels: Vec<String>,
}

/// Passes MIDI messages to the caller.
pub type MidiMessagesFn<'a> = dyn FnMut(MidiChannel, MidiMessage) + 'a;

/// Indicates that an instrument knows about MIDI.
pub trait HandlesMidi {
    /// Takes standard MIDI messages and optionally produces more in response.
    /// For example, an arpeggiator might produce notes in response to a note-on
    /// message.
    ///
    /// This method provides no way for a device to produce [WorkEvent::Control]
    /// events. If it needs to do this, it can send them at the next
    /// [Controls::work()].
    #[allow(missing_docs)]
    #[allow(unused_variables)]
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        midi_messages_fn: &mut MidiMessagesFn,
    ) {
    }

    /// Provides [MidiNoteLabelMetadata] to describe the notes an instrument
    /// supports. The caller promises to be smart about caching the results, so
    /// it's OK to generate this struct on the fly each time.
    ///
    /// Returning None means that this instrument responds to notes 0-127, and
    /// that they should be labeled according to standard musical notes (C, D,
    /// E#, etc.).
    fn midi_note_label_metadata(&self) -> Option<MidiNoteLabelMetadata> {
        None
    }
}

/// A convenience struct for consumers of [Generates]. This buffer ensures that
/// capacity and len, in Vec terms, are always the same. We call it "size."
#[derive(Debug, Default, Clone, PartialEq)]
pub struct GenerationBuffer<V: Default + Clone + std::ops::AddAssign> {
    vec: Vec<V>,
}
impl<V: Default + Clone + Copy + std::ops::AddAssign> GenerationBuffer<V> {
    /// Returns the current size of the buffer.
    pub fn buffer_size(&self) -> usize {
        self.vec.len()
    }

    /// Sets the buffer size. Does nothing if the buffer is already this size.
    pub fn resize(&mut self, size: usize) {
        if size != self.buffer_size() {
            self.vec.resize(size, V::default());
        }
    }

    /// Returns a reference to the buffer.
    pub fn buffer(&self) -> &[V] {
        &self.vec
    }

    /// Returns a mutable reference to the buffer.
    pub fn buffer_mut(&mut self) -> &mut [V] {
        &mut self.vec
    }

    /// Sets the buffer's contents to the default value. Does not change its size.
    pub fn clear(&mut self) {
        self.vec.fill(V::default());
    }

    /// Merges (adds) a slice of the same size/type to this one.
    pub fn merge(&mut self, other: &[V]) {
        assert_eq!(self.buffer_size(), other.len());
        for (src, dst) in other.iter().zip(self.buffer_mut().iter_mut()) {
            *dst += *src;
        }
    }

    /// Creates a buffer of the specified size.
    pub fn new_with(size: usize) -> Self {
        let mut r = GenerationBuffer::default();
        r.resize(size);
        r
    }
}

/// Something that [Generates] creates the given type `<V>` as its work product
/// over time. Examples are envelopes, which produce a [Normal] signal, and
/// oscillators, which produce a [BipolarNormal] signal.
#[allow(unused_variables)]
pub trait Generates<V: Default + Clone>: Send + core::fmt::Debug + Configurable {
    /// Fills a batch of values with new signal. Returns true if the signal was
    /// non-default; for example, in the case of a [StereoSample] signal,
    /// returns true if any part of the generated signal was non-silent.
    fn generate(&mut self, values: &mut [V]) -> bool {
        values.fill(V::default());
        false
    }
}

/// A [TransformsAudio] takes input audio, which is typically produced by
/// [Generates], does something to it, and then outputs it. It's what effects
/// do.
pub trait TransformsAudio: core::fmt::Debug {
    /// Transforms a buffer of audio.
    fn transform(&mut self, samples: &mut [StereoSample]) {
        for sample in samples {
            *sample = StereoSample(
                self.transform_channel(0, sample.0),
                self.transform_channel(1, sample.1),
            )
        }
    }

    /// channel: 0 is left, 1 is right. Use the value as an index into arrays.
    #[allow(unused_variables)]
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        // Default implementation is passthrough
        input_sample
    }
}

/// Each app should have a Settings struct that is composed of subsystems having
/// their own settings. Implementing [HasSettings] helps the composed struct
/// manage its parts.
pub trait HasSettings {
    /// Whether the current state of this struct has been saved to disk.
    fn has_been_saved(&self) -> bool;
    /// Call this whenever the struct changes.
    fn needs_save(&mut self);
    /// Call this after a load() or a save().
    fn mark_clean(&mut self);
}

/// An [Entity] is a generic musical instrument, which includes MIDI
/// instruments like synths, effects like reverb, and controllers like MIDI
/// sequencers. Almost everything in this system is an Entity of some kind. A
/// struct's implementation of these trait methods is usually generated by the
/// [IsEntity](ensnare_proc_macros::IsEntity) proc macro.
#[typetag::serde(tag = "type")]
pub trait Entity:
    HasMetadata
    + Controls
    + Controllable
    + Displays
    + Generates<StereoSample>
    + HandlesMidi
    + Serializable
    + TransformsAudio
    + core::fmt::Debug
    + Send
    + Sync
{
}

/// Records and replays MIDI events.
///
/// This trait does not specify behavior in case of duplicate events, which
/// allows simple implementations to use plain vectors rather than sets.
pub trait SequencesMidi: Controls + Configurable + HandlesMidi {
    /// Records a [MidiMessage] at the given [MusicalTime] on the given
    /// [MidiChannel].
    fn record_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        time: MusicalTime,
    ) -> anyhow::Result<()> {
        self.record_midi_event(channel, MidiEvent { message, time })
    }

    /// Records a [MidiEvent] on the given [MidiChannel].
    fn record_midi_event(&mut self, channel: MidiChannel, event: MidiEvent) -> anyhow::Result<()>;

    /// Removes all recorded messages.
    fn clear(&mut self);

    /// Deletes all recorded [MidiMessage]s matching the provided paramaters.
    fn remove_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        time: MusicalTime,
    ) -> anyhow::Result<()> {
        self.remove_midi_event(channel, MidiEvent { message, time })
    }

    /// Deletes all recorded [MidiEvent]s matching the provided paramaters.
    fn remove_midi_event(&mut self, channel: MidiChannel, event: MidiEvent) -> anyhow::Result<()>;

    /// Starts recording. Messages received through
    /// [HandlesMidi::handle_midi_message()] will be recorded as of the start of
    /// the time slice provided by [Controls::update_time_range()].
    ///
    /// [Controls::stop()] stops recording.
    fn start_recording(&mut self);
    /// Returns whether the sequencer is recording.
    fn is_recording(&self) -> bool;
}

/// Records and replays the given musical unit. This is another convenience
/// trait that helps rationalize sequencer interfaces while the concept of a
/// sequencer itself is under development. TODO: delete this trait when
/// sequencing is better developed.
pub trait Sequences: Controls + core::fmt::Debug {
    /// "Musical Unit"
    type MU;

    /// Records an MU to the given [MidiChannel] as of the given [MusicalTime].
    /// An MU normally lasts longer than a single point in [MusicalTime]. In
    /// such a case, `position` indicates the start of the MU, and any durations
    /// or time offsets in the MU are interpreted relative to `position`.
    fn record(
        &mut self,
        channel: MidiChannel,
        unit: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()>;

    /// Deletes all recorded MUs matching the provided paramaters.
    fn remove(
        &mut self,
        channel: MidiChannel,
        unit: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()>;

    /// Removes all recorded MUs.
    fn clear(&mut self);
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::types::CrossbeamChannel;
    use crossbeam::channel::Select;
    use std::time::Duration;

    #[derive(Debug)]
    enum TestServiceInput {
        Add(u8, u8),
    }

    #[derive(Debug, PartialEq)]
    enum TestServiceEvent {
        Added(u8),
    }

    #[derive(Debug)]
    struct TestService {
        inputs: CrossbeamChannel<TestServiceInput>,
        events: CrossbeamChannel<TestServiceEvent>,
    }
    impl Default for TestService {
        fn default() -> Self {
            let r = Self {
                inputs: Default::default(),
                events: Default::default(),
            };

            let receiver = r.inputs.receiver.clone();
            let sender = r.events.sender.clone();
            std::thread::spawn(move || {
                while let Ok(input) = receiver.recv() {
                    match input {
                        TestServiceInput::Add(a, b) => {
                            let _ = sender.send(TestServiceEvent::Added(a + b));
                        }
                    }
                }
            });

            r
        }
    }
    impl ProvidesService<TestServiceInput, TestServiceEvent> for TestService {
        fn sender(&self) -> &Sender<TestServiceInput> {
            &self.inputs.sender
        }

        fn receiver(&self) -> &Receiver<TestServiceEvent> {
            &self.events.receiver
        }
    }

    #[test]
    fn provides_service() {
        let s = TestService::default();
        let _ = s.send_input(TestServiceInput::Add(1, 2));

        let mut sel = Select::default();

        let test_receiver = s.receiver().clone();
        let test_index = sel.recv(&test_receiver);

        loop {
            match sel.select_timeout(Duration::from_secs(1)) {
                Ok(oper) => match oper.index() {
                    index if index == test_index => {
                        if let Ok(input) = TestService::recv_operation(oper, &test_receiver) {
                            match input {
                                TestServiceEvent::Added(sum) => {
                                    assert_eq!(sum, 3);
                                    break;
                                }
                            }
                        }
                    }
                    other => {
                        panic!("Unexpected select index: {other}");
                    }
                },
                Err(e) => {
                    panic!("select failed: {e:?}");
                }
            }
        }
    }

    pub(crate) fn test_trait_configurable(mut c: impl Configurable) {
        assert_ne!(
            c.sample_rate().0,
            0,
            "Default sample rate should be reasonable"
        );
        let new_sample_rate = SampleRate(3);
        c.update_sample_rate(new_sample_rate);
        assert_eq!(
            c.sample_rate(),
            new_sample_rate,
            "Sample rate should be settable"
        );

        assert!(c.tempo().0 > 0.0, "Default tempo should be reasonable");
        let new_tempo = Tempo(64.0);
        c.update_tempo(new_tempo);
        assert_eq!(c.tempo(), new_tempo, "Tempo should be settable");

        assert_eq!(
            c.time_signature(),
            TimeSignature::default(),
            "time signature should match default"
        );
        let new_time_signature = TimeSignature::new_with(13, 512).unwrap();
        assert_ne!(new_time_signature, TimeSignature::default());
        c.update_time_signature(new_time_signature);
        assert_eq!(
            c.time_signature(),
            new_time_signature,
            "Time signature should be settable"
        );
    }
}
