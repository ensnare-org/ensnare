// Copyright (c) 2024 Mike Tsao

use crate::{prelude::*, traits::Configurables};
use delegate::delegate;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// Describes the transformation that takes place to convert the signal into the
/// control event.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalPassthroughType {
    #[default]
    /// Maps -1.0..=1.0 to 0.0..=1.0. Min amplitude becomes 0.0, silence becomes
    /// 0.5, and max amplitude becomes 1.0.
    Compressed,

    /// Based on the absolute value of the sample. Silence is 0.0, and max
    /// amplitude of either polarity is 1.0.
    Amplitude,

    /// Based on the absolute value of the sample. Silence is 1.0, and max
    /// amplitude of either polarity is 0.0.
    AmplitudeInverted,
}

/// Uses an input signal as a control source. Transformation depends on
/// configuration. Uses the standard Sample::from(StereoSample) methodology of
/// averaging the two channels to create a single signal.
#[derive(Clone, Builder, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SignalPassthroughControllerCore {
    /// Which kind of transformation takes place.
    #[builder(default)]
    passthrough_type: SignalPassthroughType,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: SignalPassthroughControllerEphemerals,
}
#[derive(Clone, Debug, Default)]
pub struct SignalPassthroughControllerEphemerals {
    control_value: ControlValue,
    // We don't issue consecutive identical events, so we need to remember
    // whether we've sent the current value.
    has_value_been_issued: bool,

    is_performing: bool,

    c: Configurables,
}
impl SignalPassthroughControllerCoreBuilder {
    /// Returns a default Builder with SignalPassthroughType::Compressed
    pub fn compressed() -> Self {
        Self {
            passthrough_type: Some(SignalPassthroughType::Compressed),
            ..Default::default()
        }
    }
    /// Returns a default Builder with SignalPassthroughType::Amplitude
    pub fn amplitude() -> Self {
        Self {
            passthrough_type: Some(SignalPassthroughType::Amplitude),
            ..Default::default()
        }
    }
    /// Returns a default Builder with SignalPassthroughType::AmplitudeInverted
    pub fn amplitude_inverted() -> Self {
        Self {
            passthrough_type: Some(SignalPassthroughType::AmplitudeInverted),
            ..Default::default()
        }
    }
}
impl Serializable for SignalPassthroughControllerCore {}
impl Configurable for SignalPassthroughControllerCore {
    delegate! {
        to self.e.c {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl Controls for SignalPassthroughControllerCore {
    fn update_time_range(&mut self, _range: &TimeRange) {
        // We can ignore because we already have our own de-duplicating logic.
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if !self.e.is_performing {
            return;
        }
        if !self.e.has_value_been_issued {
            self.e.has_value_been_issued = true;
            control_events_fn(WorkEvent::Control(self.e.control_value))
        }
    }

    fn is_finished(&self) -> bool {
        true
    }

    fn play(&mut self) {
        self.e.is_performing = true;
    }

    fn stop(&mut self) {
        self.e.is_performing = false;
    }

    fn skip_to_start(&mut self) {}
}
impl HandlesMidi for SignalPassthroughControllerCore {}
impl TransformsAudio for SignalPassthroughControllerCore {
    fn transform(&mut self, samples: &mut [StereoSample]) {
        for sample in samples {
            let mono_sample: Sample = (*sample).into();
            let control_value = match self.passthrough_type {
                SignalPassthroughType::Compressed => {
                    let as_bipolar_normal: BipolarNormal = mono_sample.into();
                    as_bipolar_normal.into()
                }
                SignalPassthroughType::Amplitude => ControlValue(mono_sample.0.abs()),
                SignalPassthroughType::AmplitudeInverted => ControlValue(1.0 - mono_sample.0.abs()),
            };
            if self.e.control_value != control_value {
                self.e.has_value_been_issued = false;
                self.e.control_value = control_value;
            }

            // We don't alter the input. We just look at it.
        }
    }
}
