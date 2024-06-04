// Copyright (c) 2024 Mike Tsao

//! The traits that define many characteristics and relationships among parts of
//! the system.

use crate::{prelude::*, types::MidiMessage};
use crossbeam::channel::{Receiver, Sender};

/// Quick import of all important traits.
pub mod prelude {
    #[cfg(feature = "egui")]
    pub use super::Displays;
    pub use super::{
        Configurable,
        Configurables,
        Serializable,
        WorkEvent,
        // CanPrototype, ControlEventsFn, ControlProxyEventsFn, Controllable,
        // Controls, ControlsAsProxy, DisplaysAction, EntityBounds, Generates,
        // GeneratesEnvelope, GenerationBuffer, HandlesMidi, HasExtent,
        // HasMetadata, HasSettings, IsStereoSampleVoice, IsVoice,
        // MidiMessagesFn, PlaysNotes, Sequences, SequencesMidi, StoresVoices,
        // TransformsAudio,
    };
}

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

#[cfg(test)]
mod tests {
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
