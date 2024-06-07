// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use delegate::delegate;
use derivative::Derivative;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// TODO: pub(crate)
#[allow(missing_docs)]
pub trait Delays {
    fn peek_output(&self, apply_decay: bool) -> Sample;
    fn peek_indexed_output(&self, index: isize) -> Sample;
    fn pop_output(&mut self, input: Sample) -> Sample;
}

/// TODO: pub(crate)
#[derive(Clone, Debug, Derivative)]
#[derivative(Default)]
pub struct DelayLine {
    #[derivative(Default(value = "0.1.into()"))]
    delay: Seconds,
    #[derivative(Default(value = "0.5.into()"))]
    decay: Normal,

    buffer_size: usize,
    buffer_pointer: usize,
    buffer: Vec<Sample>,

    c: Configurables,
}
impl Configurable for DelayLine {
    delegate! {
        to self.c {
            fn sample_rate(&self) -> SampleRate;
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.c.update_sample_rate(sample_rate);
        self.resize_buffer();
    }
}
#[allow(missing_docs)]
impl DelayLine {
    /// decay: 1.0 = no decay
    pub fn new_with(delay: Seconds, decay: Normal) -> Self {
        let mut r = Self {
            delay,
            decay,
            ..Default::default()
        };
        r.resize_buffer();
        r
    }

    pub fn delay(&self) -> Seconds {
        self.delay
    }

    pub fn set_delay(&mut self, delay: Seconds) {
        if delay != self.delay {
            self.delay = delay;
            self.resize_buffer();
        }
    }

    fn resize_buffer(&mut self) {
        self.buffer_size = (self.c.sample_rate() * self.delay).into();
        self.buffer = Vec::with_capacity(self.buffer_size);
        self.buffer.resize(self.buffer_size, Sample::SILENCE);
        self.buffer_pointer = 0;
    }

    pub fn decay(&self) -> Normal {
        self.decay
    }

    pub fn set_decay(&mut self, decay: Normal) {
        self.decay = decay;
    }
}
impl Delays for DelayLine {
    fn peek_output(&self, apply_decay: bool) -> Sample {
        if self.buffer_size == 0 {
            Sample::SILENCE
        } else if apply_decay {
            self.buffer[self.buffer_pointer] * self.decay()
        } else {
            self.buffer[self.buffer_pointer]
        }
    }

    fn peek_indexed_output(&self, index: isize) -> Sample {
        if self.buffer_size == 0 {
            Sample::SILENCE
        } else {
            let mut index = -index;
            while index < 0 {
                index += self.buffer_size as isize;
            }
            self.buffer[self.buffer_pointer]
        }
    }

    fn pop_output(&mut self, input: Sample) -> Sample {
        if self.buffer_size == 0 {
            input
        } else {
            let out = self.peek_output(true);
            self.buffer[self.buffer_pointer] = input;
            self.buffer_pointer += 1;
            if self.buffer_pointer >= self.buffer_size {
                self.buffer_pointer = 0;
            }
            out
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RecirculatingDelayLine {
    delay: DelayLine,
}
impl RecirculatingDelayLine {
    pub(crate) fn new_with(
        delay: Seconds,
        decay: Seconds,
        final_amplitude: Normal,
        peak_amplitude: Normal,
    ) -> Self {
        Self {
            delay: DelayLine::new_with(
                delay,
                (peak_amplitude.0 * final_amplitude.0)
                    .powf((delay / decay).into())
                    .into(),
            ),
        }
    }

    pub(super) fn decay_factor(&self) -> Normal {
        self.delay.decay()
    }
}
impl Delays for RecirculatingDelayLine {
    fn peek_output(&self, apply_decay: bool) -> Sample {
        self.delay.peek_output(apply_decay)
    }

    fn peek_indexed_output(&self, index: isize) -> Sample {
        self.delay.peek_indexed_output(index)
    }

    fn pop_output(&mut self, input: Sample) -> Sample {
        let output = self.peek_output(true);
        self.delay.pop_output(input + output);
        output
    }
}
impl Configurable for RecirculatingDelayLine {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.delay.update_sample_rate(sample_rate);
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct AllPassDelayLine {
    delay: RecirculatingDelayLine,
}
impl AllPassDelayLine {
    pub(crate) fn new_with(
        delay: Seconds,
        decay: Seconds,
        final_amplitude: Normal,
        peak_amplitude: Normal,
    ) -> Self {
        Self {
            delay: RecirculatingDelayLine::new_with(delay, decay, final_amplitude, peak_amplitude),
        }
    }
}
impl Delays for AllPassDelayLine {
    fn peek_output(&self, _apply_decay: bool) -> Sample {
        panic!("AllPassDelay doesn't allow peeking")
    }

    fn peek_indexed_output(&self, _: isize) -> Sample {
        panic!("AllPassDelay doesn't allow peeking")
    }

    fn pop_output(&mut self, input: Sample) -> Sample {
        let decay_factor = self.delay.decay_factor();
        let vm = self.delay.peek_output(false);
        let vn = input - (vm * decay_factor);
        self.delay.pop_output(vn);
        vm + vn * decay_factor
    }
}
impl Configurable for AllPassDelayLine {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.delay.update_sample_rate(sample_rate)
    }
}

/// A delay effect.
#[derive(Clone, Debug, Builder, Derivative, Control, Deserialize, Serialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct DelayCore {
    /// The number of seconds of delay.
    #[control]
    seconds: Seconds,

    /// How fast the signal decays.
    #[control]
    decay: Normal,

    #[serde(skip)]
    #[builder(setter(skip))]
    delay: DelayLine,
}
impl DelayCoreBuilder {
    /// The overridden Builder build() method.
    pub fn build(&self) -> Result<DelayCore, DelayCoreBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }
}
impl Serializable for DelayCore {
    fn after_deser(&mut self) {
        self.delay = DelayLine::new_with(self.seconds, self.decay)
    }
}
impl Configurable for DelayCore {
    delegate! {
        to self.delay {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl TransformsAudio for DelayCore {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        self.delay.pop_output(input_sample)
    }
}
impl DelayCore {
    #[allow(missing_docs)]
    pub fn seconds(&self) -> Seconds {
        self.delay.delay()
    }

    #[allow(missing_docs)]
    pub fn set_seconds(&mut self, seconds: Seconds) {
        self.delay.set_delay(seconds);
    }

    #[allow(missing_docs)]
    pub fn decay(&self) -> Normal {
        self.decay
    }

    #[allow(missing_docs)]
    pub fn set_decay(&mut self, decay: Normal) {
        self.delay.set_decay(decay);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;
    use more_asserts::{assert_gt, assert_lt};

    // This small rate allows us to observe expected behavior after a small
    // number of iterations.
    const CURIOUSLY_SMALL_SAMPLE_RATE: SampleRate = SampleRate::new(3);

    #[test]
    fn basic_delay() {
        let mut fx = DelayCoreBuilder::default()
            .seconds(1.0.into())
            .build()
            .unwrap();
        fx.update_sample_rate(SampleRate::DEFAULT);

        // Add a unique first sample.
        assert_eq!(fx.transform_channel(0, Sample::from(0.5)), Sample::SILENCE);

        // Push a whole bunch more.
        for i in 0..SampleRate::DEFAULT_SAMPLE_RATE - 1 {
            assert_eq!(
                fx.transform_channel(0, Sample::MAX),
                Sample::SILENCE,
                "unexpected value at sample {}",
                i
            );
        }

        // We should get back our first sentinel sample.
        assert_eq!(fx.transform_channel(0, Sample::SILENCE), Sample::from(0.5));

        // And the next should be one of the bunch.
        assert_eq!(fx.transform_channel(0, Sample::SILENCE), Sample::MAX);
    }

    #[test]
    fn delay_zero() {
        let mut fx = DelayCoreBuilder::default()
            .seconds(0.0.into())
            .build()
            .unwrap();
        fx.update_sample_rate(SampleRate::DEFAULT);

        // We should keep getting back what we put in.
        let mut rng = Rng::new_with_seed(12345u128);
        for i in 0..SampleRate::DEFAULT_SAMPLE_RATE {
            let random_bipolar_normal = rng.rand_float() * 2.0 - 1.0;
            let sample = Sample::from(random_bipolar_normal);
            assert_eq!(
                fx.transform_channel(0, sample),
                sample,
                "unexpected value at sample {}",
                i
            );
        }
    }

    #[test]
    fn delay_line() {
        // It's very simple: it should return an input sample, attenuated, after
        // the specified delay.
        let mut delay = DelayLine::new_with(1.0.into(), 0.3.into());
        delay.update_sample_rate(CURIOUSLY_SMALL_SAMPLE_RATE);

        assert_eq!(delay.pop_output(Sample::from(0.5)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.4)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.3)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.2)), Sample::from(0.5 * 0.3));
    }

    #[test]
    fn recirculating_delay_line() {
        // Recirculating means that the input value is added to the value at the
        // back of the buffer, rather than replacing that value. So if we put in
        // a single value, we should expect to get it back, usually quieter,
        // each time it cycles through the buffer.
        let mut delay = RecirculatingDelayLine::new_with(
            1.0.into(),
            1.5.into(),
            Normal::from(0.001),
            Normal::from(1.0),
        );
        delay.update_sample_rate(CURIOUSLY_SMALL_SAMPLE_RATE);

        assert_eq!(delay.pop_output(Sample::from(0.5)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert!(approx_eq!(
            SampleType,
            delay.pop_output(Sample::from(0.0)).0,
            Sample::from(0.5 * 0.01).0,
            epsilon = 0.001
        ));
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert!(approx_eq!(
            SampleType,
            delay.pop_output(Sample::from(0.0)).0,
            Sample::from(0.5 * 0.01 * 0.01).0,
            epsilon = 0.001
        ));
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
    }

    #[test]
    fn allpass_delay_line() {
        // TODO: I'm not sure what this delay line is supposed to do.
        let mut delay = AllPassDelayLine::new_with(
            1.0.into(),
            1.5.into(),
            Normal::from(0.001),
            Normal::from(1.0),
        );
        delay.update_sample_rate(CURIOUSLY_SMALL_SAMPLE_RATE);

        assert_lt!(delay.pop_output(Sample::from(0.5)), Sample::from(0.5));
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert_eq!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE);
        assert_gt!(delay.pop_output(Sample::from(0.0)), Sample::SILENCE); // Note! > not =
    }
}
