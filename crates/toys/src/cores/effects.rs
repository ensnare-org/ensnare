// Copyright (c) 2024 Mike Tsao

use ensnare::prelude::*;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// An effect that applies a negative gain.
#[derive(Debug, Default, Control, Serialize, Deserialize)]
pub struct ToyEffectCore {
    /// This effect transformation is signal * -magnitude.
    #[control]
    pub magnitude: Normal,

    sample_rate: SampleRate,
    tempo: Tempo,
    time_signature: TimeSignature,
}
impl ToyEffectCore {
    pub fn new_with(magnitude: Normal) -> Self {
        Self {
            magnitude,
            ..Default::default()
        }
    }

    pub fn set_magnitude(&mut self, magnitude: Normal) {
        self.magnitude = magnitude;
    }

    #[allow(dead_code)]
    pub fn magnitude(&self) -> Normal {
        self.magnitude
    }
}
impl TransformsAudio for ToyEffectCore {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        input_sample * self.magnitude * -1.0
    }
}
impl Configurable for ToyEffectCore {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
    }

    fn tempo(&self) -> Tempo {
        self.tempo
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.tempo = tempo;
    }

    fn time_signature(&self) -> TimeSignature {
        self.time_signature
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.time_signature = time_signature;
    }
}
impl Serializable for ToyEffectCore {}
impl HandlesMidi for ToyEffectCore {}
