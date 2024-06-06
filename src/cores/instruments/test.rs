// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use delegate::delegate;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// Produces a constant audio signal. Used for ensuring that a known signal
/// value gets all the way through the pipeline.
#[derive(Clone, Builder, Debug, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(default)]
pub struct TestAudioSourceCore {
    /// The value of the constant audio signal.
    #[control]
    // This should be a Normal, but we use this audio source for testing edge
    // conditions. Thus we need to let it go out of range.
    level: ParameterType,

    #[serde(skip)]
    #[builder(setter(skip))]
    c: Configurables,
}
impl Generates<StereoSample> for TestAudioSourceCore {
    fn generate(&mut self, values: &mut [StereoSample]) -> bool {
        values.fill(StereoSample::from(self.level));
        self.level != 0.0
    }
}
impl Configurable for TestAudioSourceCore {
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
impl TestAudioSourceCore {
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
}
