// Copyright (c) 2024 Mike Tsao

use super::delay::{AllPassDelayLine, Delays, RecirculatingDelayLine};
use crate::prelude::*;
use delegate::delegate;
use derivative::Derivative;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// Schroeder reverb. Uses four parallel recirculating delay lines feeding into
/// a series of two all-pass delay lines.
#[derive(Debug, Derivative, Control, Builder, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct ReverbCore {
    /// How much the effect should attenuate the input.
    #[control]
    #[derivative(Default(value = "0.8.into()"))]
    attenuation: Normal,

    /// The time value that determines the delay-line configuration.
    #[control]
    #[derivative(Default(value = "1.0.into()"))]
    seconds: Seconds,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: ReverbCoreEphemerals,
}
#[derive(Clone, Debug, Default)]
pub struct ReverbCoreEphemerals {
    channels: [ReverbChannel; 2],
    c: Configurables,
}
impl ReverbCoreBuilder {
    /// The overridden Builder build() method.
    pub fn build(&self) -> Result<ReverbCore, ReverbCoreBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }
}
impl Serializable for ReverbCore {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        self.e.channels = [
            ReverbChannel::new_with(self.attenuation, self.seconds),
            ReverbChannel::new_with(self.attenuation, self.seconds),
        ];
    }
}
impl Configurable for ReverbCore {
    delegate! {
        to self.e.c {
            fn sample_rate(&self) -> SampleRate;
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.e.c.update_sample_rate(sample_rate);
        self.e.channels[0].update_sample_rate(sample_rate);
        self.e.channels[1].update_sample_rate(sample_rate);
    }
}
impl TransformsAudio for ReverbCore {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        self.e.channels[channel].transform_channel(channel, input_sample)
    }
}
impl ReverbCore {
    #[allow(missing_docs)]
    pub fn attenuation(&self) -> Normal {
        self.attenuation
    }

    #[allow(missing_docs)]
    pub fn set_attenuation(&mut self, attenuation: Normal) {
        self.attenuation = attenuation;
        self.e
            .channels
            .iter_mut()
            .for_each(|c| c.set_attenuation(attenuation));
    }

    #[allow(missing_docs)]
    pub fn seconds(&self) -> Seconds {
        self.seconds
    }

    #[allow(missing_docs)]
    pub fn set_seconds(&mut self, seconds: Seconds) {
        self.seconds = seconds;
        self.e
            .channels
            .iter_mut()
            .for_each(|c| c.set_seconds(seconds));
    }
}

#[derive(Clone, Debug, Default)]
struct ReverbChannel {
    attenuation: Normal,

    recirc_delay_lines: Vec<RecirculatingDelayLine>,
    allpass_delay_lines: Vec<AllPassDelayLine>,
}
impl TransformsAudio for ReverbChannel {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        let input_attenuated = input_sample * self.attenuation.0;
        let recirc_output = self.recirc_delay_lines[0].pop_output(input_attenuated)
            + self.recirc_delay_lines[1].pop_output(input_attenuated)
            + self.recirc_delay_lines[2].pop_output(input_attenuated)
            + self.recirc_delay_lines[3].pop_output(input_attenuated);
        let adl_0_out = self.allpass_delay_lines[0].pop_output(recirc_output);
        self.allpass_delay_lines[1].pop_output(adl_0_out)
    }
}
impl Serializable for ReverbChannel {}
impl Configurable for ReverbChannel {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.recirc_delay_lines
            .iter_mut()
            .for_each(|r| r.update_sample_rate(sample_rate));
        self.allpass_delay_lines
            .iter_mut()
            .for_each(|r| r.update_sample_rate(sample_rate));
    }
}
impl ReverbChannel {
    pub fn new_with(attenuation: Normal, seconds: Seconds) -> Self {
        // Thanks to https://basicsynth.com/ (page 133 of paperback) for
        // constants.
        Self {
            attenuation,
            recirc_delay_lines: Self::instantiate_recirc_delay_lines(seconds),
            allpass_delay_lines: Self::instantiate_allpass_delay_lines(),
        }
    }

    fn set_attenuation(&mut self, attenuation: Normal) {
        self.attenuation = attenuation;
    }

    fn set_seconds(&mut self, seconds: Seconds) {
        self.recirc_delay_lines = Self::instantiate_recirc_delay_lines(seconds);
    }

    fn instantiate_recirc_delay_lines(seconds: Seconds) -> Vec<RecirculatingDelayLine> {
        // Thanks to https://basicsynth.com/ (page 133 of paperback) for
        // constants.
        vec![
            RecirculatingDelayLine::new_with(
                0.0297.into(),
                seconds,
                Normal::from(0.001),
                Normal::from(1.0),
            ),
            RecirculatingDelayLine::new_with(
                0.0371.into(),
                seconds,
                Normal::from(0.001),
                Normal::from(1.0),
            ),
            RecirculatingDelayLine::new_with(
                0.0411.into(),
                seconds,
                Normal::from(0.001),
                Normal::from(1.0),
            ),
            RecirculatingDelayLine::new_with(
                0.0437.into(),
                seconds,
                Normal::from(0.001),
                Normal::from(1.0),
            ),
        ]
    }

    fn instantiate_allpass_delay_lines() -> Vec<AllPassDelayLine> {
        vec![
            AllPassDelayLine::new_with(
                0.09683.into(),
                0.0050.into(),
                Normal::from(0.001),
                Normal::from(1.0),
            ),
            AllPassDelayLine::new_with(
                0.03292.into(),
                0.0017.into(),
                Normal::from(0.001),
                Normal::from(1.0),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverb_does_anything_at_all() {
        // This test is lame, because I can't think of a programmatic way to
        // test that reverb works. I observed that with the Schroeder reverb set
        // to 0.5 seconds, we start getting back nonzero samples (first
        // 0.47767496) at samples: 29079, seconds: 0.65938777. This doesn't look
        // wrong, but I couldn't have predicted that exact number.
        let mut fx = ReverbCoreBuilder::default()
            .attenuation(0.9.into())
            .seconds(0.5.into())
            .build()
            .unwrap();
        assert_eq!(fx.transform_channel(0, Sample::from(0.8)), Sample::SILENCE);
        let mut s = Sample::default();
        for _ in 0..SampleRate::DEFAULT.0 {
            s += fx.transform_channel(0, Sample::SILENCE);
        }
        assert!(s != Sample::SILENCE);

        // TODO: this test might not do anything. I refactored it in a hurry and
        // took something that looked critical (skipping the clock to 0.5
        // seconds) out of it, but it still passed. I might not actually be
        // testing anything useful.
    }
}
