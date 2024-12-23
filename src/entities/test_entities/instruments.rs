// Copyright (c) 2024 Mike Tsao

use crate::{cores::SimpleConstantAudioSourceCore, prelude::*};
use ensnare_proc_macros::{Control, InnerControllable, InnerInstrument, IsEntity, Metadata};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Emits a constant audio value
#[derive(
    Debug, Default, InnerControllable, InnerInstrument, IsEntity, Metadata, Serialize, Deserialize,
)]
#[entity(
    Configurable,
    Controls,
    Displays,
    HandlesMidi,
    Serializable,
    SkipInner,
    TransformsAudio
)]
#[serde(rename_all = "kebab-case")]
pub struct TestAudioSource {
    uid: Uid,
    inner: SimpleConstantAudioSourceCore,
}
#[allow(missing_docs)]
impl TestAudioSource {
    pub const TOO_LOUD: SampleType = 1.1;
    pub const LOUD: SampleType = 1.0;
    pub const MEDIUM: SampleType = 0.5;
    pub const SILENT: SampleType = 0.0;
    pub const QUIET: SampleType = -1.0;
    pub const TOO_QUIET: SampleType = -1.1;

    pub fn new_with(uid: Uid, inner: SimpleConstantAudioSourceCore) -> Self {
        Self { uid, inner }
    }
}

/// The smallest possible [IsEntity] that acts like an instrument.
#[derive(Debug, Default, Control, IsEntity, Metadata, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[entity(
    Controls,
    Displays,
    HandlesMidi,
    Serializable,
    SkipInner,
    TransformsAudio
)]
pub struct TestInstrument {
    #[allow(missing_docs)]
    pub uid: Uid,
    #[allow(missing_docs)]
    pub sample_rate: SampleRate,

    #[allow(missing_docs)]
    #[control]
    pub dca: Dca,
}
impl TestInstrument {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            ..Default::default()
        }
    }

    fn notify_change_dca(&mut self) {}
}
impl Configurable for TestInstrument {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
    }
}
impl Generates<StereoSample> for TestInstrument {}

/// An [IsEntity] that counts how many MIDI messages it has received.
#[derive(Debug, Default, Control, IsEntity, Metadata, Serialize, Deserialize)]
#[entity(
    Configurable,
    Controls,
    Displays,
    Serializable,
    SkipInner,
    TransformsAudio
)]
#[serde(rename_all = "kebab-case")]
pub struct TestInstrumentCountsMidiMessages {
    uid: Uid,

    #[allow(missing_docs)]
    #[serde(skip)]
    pub received_midi_message_count: Arc<Mutex<usize>>,
}
impl Generates<StereoSample> for TestInstrumentCountsMidiMessages {}
impl HandlesMidi for TestInstrumentCountsMidiMessages {
    fn handle_midi_message(
        &mut self,
        _action: MidiChannel,
        _: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        if let Ok(mut received_count) = self.received_midi_message_count.lock() {
            *received_count += 1;
        }
    }
}
impl TestInstrumentCountsMidiMessages {
    #[allow(missing_docs)]
    pub fn received_midi_message_count_mutex(&self) -> &Arc<Mutex<usize>> {
        &self.received_midi_message_count
    }
}
