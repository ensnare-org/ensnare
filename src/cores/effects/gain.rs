// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use delegate::delegate;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// An effect that multiplies the signal by a constant factor.
#[derive(Debug, Builder, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(default)]
pub struct GainCore {
    /// The multiplier that is applied to each sample.
    #[control]
    ceiling: Normal,

    #[serde(skip)]
    #[builder(setter(skip))]
    c: Configurables,
}
impl Serializable for GainCore {}
impl Configurable for GainCore {
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
impl TransformsAudio for GainCore {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        Sample(input_sample.0 * self.ceiling.0)
    }
}
impl GainCore {
    #[allow(missing_docs)]
    pub fn ceiling(&self) -> Normal {
        self.ceiling
    }

    #[allow(missing_docs)]
    pub fn set_ceiling(&mut self, ceiling: Normal) {
        self.ceiling = ceiling;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cores::instruments::{TestAudioSourceCore, TestAudioSourceCoreBuilder};

    #[test]
    fn gain_mainline() {
        let mut gain = GainCoreBuilder::default()
            .ceiling(0.5.into())
            .build()
            .unwrap();
        let mut buffer = [StereoSample::default(); 1];
        TestAudioSourceCoreBuilder::default()
            .level(TestAudioSourceCore::LOUD)
            .build()
            .unwrap()
            .generate(&mut buffer);
        gain.transform(&mut buffer);
        assert_eq!(buffer[0], StereoSample::from(0.5));
    }
}
