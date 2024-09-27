// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use delegate::delegate;
use derivative::Derivative;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// Clamps an audio signal to the configured min/max.
#[derive(Debug, Builder, Derivative, Control, Serialize, Deserialize)]
#[derivative(Default)]
#[builder(default)]
#[serde(rename_all = "kebab-case")]
pub struct LimiterCore {
    /// The minimum value the limiter will allow.
    #[control]
    #[derivative(Default(value = "Normal::minimum()"))]
    minimum: Normal,

    /// The maximum value the limiter will allow.
    #[control]
    #[derivative(Default(value = "Normal::maximum()"))]
    maximum: Normal,

    #[serde(skip)]
    #[builder(setter(skip))]
    c: Configurables,
}
impl Serializable for LimiterCore {}
impl Configurable for LimiterCore {
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
impl TransformsAudio for LimiterCore {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        let sign = input_sample.0.signum();
        Sample::from(input_sample.0.abs().clamp(self.minimum.0, self.maximum.0) * sign)
    }
}
impl LimiterCore {
    /// The maximum value the limiter will allow.
    pub fn maximum(&self) -> Normal {
        self.maximum
    }

    #[allow(missing_docs)]
    pub fn set_maximum(&mut self, max: Normal) {
        self.maximum = max;
    }

    /// The minimum value the limiter will allow.
    pub fn minimum(&self) -> Normal {
        self.minimum
    }

    #[allow(missing_docs)]
    pub fn set_minimum(&mut self, min: Normal) {
        self.minimum = min;
    }
}

/// re-enable when moved into new crate
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cores::instruments::{TestAudioSourceCore, TestAudioSourceCoreBuilder};
    use more_asserts::{assert_gt, assert_lt};

    #[test]
    fn limiter_mainline() {
        let mut buffer = [StereoSample::default(); 1];

        // audio sources are at or past boundaries
        TestAudioSourceCoreBuilder::default()
            .level(TestAudioSourceCore::TOO_LOUD)
            .build()
            .unwrap()
            .generate(&mut buffer);
        assert_gt!(buffer[0], StereoSample::MAX);
        TestAudioSourceCoreBuilder::default()
            .level(TestAudioSourceCore::LOUD)
            .build()
            .unwrap()
            .generate(&mut buffer);
        assert_eq!(buffer[0], StereoSample::MAX);
        TestAudioSourceCoreBuilder::default()
            .level(TestAudioSourceCore::SILENT)
            .build()
            .unwrap()
            .generate(&mut buffer);
        assert_eq!(buffer[0], StereoSample::SILENCE);
        TestAudioSourceCoreBuilder::default()
            .level(TestAudioSourceCore::QUIET)
            .build()
            .unwrap()
            .generate(&mut buffer);
        assert_eq!(buffer[0], StereoSample::MIN);
        TestAudioSourceCoreBuilder::default()
            .level(TestAudioSourceCore::TOO_QUIET)
            .build()
            .unwrap()
            .generate(&mut buffer);
        assert_lt!(buffer[0], StereoSample::MIN);

        // Limiter clamps high and low, and doesn't change values inside the range.
        let mut limiter = LimiterCore::default();
        TestAudioSourceCoreBuilder::default()
            .level(TestAudioSourceCore::TOO_LOUD)
            .build()
            .unwrap()
            .generate(&mut buffer);
        limiter.transform(&mut buffer);
        assert_eq!(buffer[0], StereoSample::MAX);
        TestAudioSourceCoreBuilder::default()
            .level(TestAudioSourceCore::LOUD)
            .build()
            .unwrap()
            .generate(&mut buffer);
        limiter.transform(&mut buffer);
        assert_eq!(buffer[0], StereoSample::MAX);
        TestAudioSourceCoreBuilder::default()
            .level(TestAudioSourceCore::SILENT)
            .build()
            .unwrap()
            .generate(&mut buffer);
        limiter.transform(&mut buffer);
        assert_eq!(buffer[0], StereoSample::SILENCE);
        TestAudioSourceCoreBuilder::default()
            .level(TestAudioSourceCore::QUIET)
            .build()
            .unwrap()
            .generate(&mut buffer);
        limiter.transform(&mut buffer);
        assert_eq!(buffer[0], StereoSample::MIN);
        TestAudioSourceCoreBuilder::default()
            .level(TestAudioSourceCore::TOO_QUIET)
            .build()
            .unwrap()
            .generate(&mut buffer);
        limiter.transform(&mut buffer);
        assert_eq!(buffer[0], StereoSample::MIN);
    }

    #[test]
    fn limiter_bias() {
        let mut limiter = LimiterCoreBuilder::default()
            .minimum(0.2.into())
            .maximum(0.8.into())
            .build()
            .unwrap();
        assert_eq!(
            limiter.transform_channel(0, Sample::from(0.1)),
            Sample::from(0.2),
            "Limiter failed to clamp min {}",
            0.2
        );
        assert_eq!(
            limiter.transform_channel(0, Sample::from(0.9)),
            Sample::from(0.8),
            "Limiter failed to clamp max {}",
            0.8
        );
        assert_eq!(
            limiter.transform_channel(0, Sample::from(-0.1)),
            Sample::from(-0.2),
            "Limiter failed to clamp min {} for negative values",
            0.2
        );
        assert_eq!(
            limiter.transform_channel(0, Sample::from(-0.9)),
            Sample::from(-0.8),
            "Limiter failed to clamp max {} for negative values",
            0.8
        );
    }
}
