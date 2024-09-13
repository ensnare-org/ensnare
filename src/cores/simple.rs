// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use delegate::delegate;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// A controller that emits MIDI note-on messages every time [Controls::work()]
/// is called.
#[derive(Debug, Default, Control)]
pub struct SimpleControllerAlwaysSendsMidiMessageCore {
    midi_note: u8,
    is_performing: bool,
}
impl HandlesMidi for SimpleControllerAlwaysSendsMidiMessageCore {}
impl Controls for SimpleControllerAlwaysSendsMidiMessageCore {
    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if self.is_performing {
            control_events_fn(WorkEvent::Midi(
                MidiChannel::default(),
                MidiMessage::NoteOn {
                    key: u7::from(self.midi_note),
                    vel: u7::from(127),
                },
            ));
            self.midi_note += 1;
            if self.midi_note > 127 {
                self.midi_note = 1;
            }
        }
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }
}
impl Configurable for SimpleControllerAlwaysSendsMidiMessageCore {}
impl Serializable for SimpleControllerAlwaysSendsMidiMessageCore {}

/// An effect that multiplies the input by 0.5, which is basically a gain set to 50%.
#[derive(Debug, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SimpleEffectHalfCore {}
impl TransformsAudio for SimpleEffectHalfCore {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        input_sample * 0.5
    }
}
impl Serializable for SimpleEffectHalfCore {}
impl Configurable for SimpleEffectHalfCore {}

/// Produces a constant audio signal. Used for ensuring that a known signal
/// value gets all the way through the pipeline.
#[derive(Clone, Builder, Debug, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(default)]
pub struct SimpleAudioSourceCore {
    /// The value of the constant audio signal.
    // This should be a Normal, but we use this audio source for testing edge
    // conditions. Thus we need to let it go out of range.
    #[control]
    level: ParameterType,

    #[serde(skip)]
    #[builder(setter(skip))]
    c: Configurables,
}
impl Generates<StereoSample> for SimpleAudioSourceCore {
    fn generate(&mut self, values: &mut [StereoSample]) -> bool {
        let s = StereoSample::from(self.level);
        values.fill(s);
        self.level != 0.0
    }
}
impl Configurable for SimpleAudioSourceCore {
    delegate! {
        to self.c {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl SimpleAudioSourceCore {
    /// Higher than maximum valid positive value.
    pub const TOO_LOUD: SampleType = 1.1;
    /// Maximum valid positive value.
    pub const LOUD: SampleType = 1.0;
    /// An ordinary positive value.
    pub const MEDIUM: SampleType = 0.5;
    /// Silence.
    pub const SILENT: SampleType = 0.0;
    /// Lowest negative value.
    pub const LOUD_NEGATIVE: SampleType = -1.0;
    /// Lower than minimum valid negative value.
    pub const TOO_LOUD_NEGATIVE: SampleType = -1.1;

    /// The constant signal level of this core device.
    pub fn level(&self) -> f64 {
        self.level
    }

    /// Sets the device's signal level.
    pub fn set_level(&mut self, level: ParameterType) {
        self.level = level;
    }

    /// Initializes with the desired audio level.
    pub fn new_with(level: ParameterType) -> Self {
        Self {
            level: level,
            c: Default::default(),
        }
    }
}

/// Produces a constant audio signal consisting of random samples. Used for
/// ensuring that a known signal value gets all the way through the pipeline.
#[derive(Builder, Debug, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(default)]
pub struct SimpleNoisyAudioSourceCore {
    #[serde(skip)]
    #[builder(setter(skip))]
    c: Configurables,

    #[serde(skip)]
    #[builder(setter(skip))]
    r: Rng,
}
impl Generates<StereoSample> for SimpleNoisyAudioSourceCore {
    fn generate(&mut self, values: &mut [StereoSample]) -> bool {
        for v in values.iter_mut() {
            *v = self.r.rand_stereo_sample()
        }
        true
    }
}
impl Configurable for SimpleNoisyAudioSourceCore {
    delegate! {
        to self.c {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
