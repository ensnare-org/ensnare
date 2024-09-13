// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use core::f64::consts::PI;
use delegate::delegate;
use derivative::Derivative;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
#[derive(Debug, Clone, Derivative, Control, Builder, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct BiQuadFilterLowPass24dbCore {
    #[control]
    #[derivative(Default(value = "500.0.into()"))]
    cutoff: FrequencyHz,
    #[control]
    #[derivative(Default(value = "0.85"))]
    passband_ripple: ParameterType,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: BiQuadFilterLowPass24dbCoreEphemerals,
}
#[derive(Debug, Clone, Default)]
pub struct BiQuadFilterLowPass24dbCoreEphemerals {
    channels: [BiQuadFilterLowPass24dbChannel; 2],

    c: Configurables,
}
impl BiQuadFilterLowPass24dbCoreBuilder {
    /// The overridden Builder build() method.
    pub fn build(
        &self,
    ) -> Result<BiQuadFilterLowPass24dbCore, BiQuadFilterLowPass24dbCoreBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }
}
impl Serializable for BiQuadFilterLowPass24dbCore {
    fn after_deser(&mut self) {
        self.update_coefficients()
    }
}
impl Configurable for BiQuadFilterLowPass24dbCore {
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
        self.update_coefficients();
    }
}
impl TransformsAudio for BiQuadFilterLowPass24dbCore {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        match channel {
            0 | 1 => self.e.channels[channel].transform_channel(channel, input_sample),
            _ => panic!(),
        }
    }
}
impl BiQuadFilterLowPass24dbCore {
    fn update_coefficients(&mut self) {
        self.e.channels[0].update_coefficients(
            self.e.c.sample_rate(),
            self.cutoff,
            self.passband_ripple,
        );
        self.e.channels[1].update_coefficients(
            self.e.c.sample_rate(),
            self.cutoff,
            self.passband_ripple,
        );
    }

    #[allow(missing_docs)]
    pub fn cutoff(&self) -> FrequencyHz {
        self.cutoff
    }

    #[allow(missing_docs)]
    pub fn set_cutoff(&mut self, cutoff: FrequencyHz) {
        if self.cutoff != cutoff {
            self.cutoff = cutoff;
            self.update_coefficients();
        }
    }

    #[allow(missing_docs)]
    pub fn passband_ripple(&self) -> ParameterType {
        self.passband_ripple
    }

    #[allow(missing_docs)]
    pub fn set_passband_ripple(&mut self, passband_ripple: ParameterType) {
        if self.passband_ripple != passband_ripple {
            self.passband_ripple = passband_ripple;
            self.update_coefficients();
        }
    }
}
impl CanPrototype for BiQuadFilterLowPass24dbCore {
    fn make_another(&self) -> Self {
        let mut r = Self::default();
        r.update_from_prototype(self);
        r
    }
    fn update_from_prototype(&mut self, prototype: &Self) -> &Self {
        self.set_cutoff(prototype.cutoff());
        self.set_passband_ripple(prototype.passband_ripple());
        self
    }
}

#[derive(Debug, Clone, Default)]
struct BiQuadFilterLowPass24dbChannel {
    inner: BiQuadFilter,
    coefficients2: CoefficientSet2,
}
impl TransformsAudio for BiQuadFilterLowPass24dbChannel {
    fn transform_channel(&mut self, _: usize, input_sample: Sample) -> Sample {
        // Thanks
        // https://www.musicdsp.org/en/latest/Filters/229-lpf-24db-oct.html
        let input = input_sample.0;
        let stage_1 = self.inner.coefficients.b0 * input + self.inner.state_0;
        self.inner.state_0 = self.inner.coefficients.b1 * input
            + self.inner.coefficients.a1 * stage_1
            + self.inner.state_1;
        self.inner.state_1 =
            self.inner.coefficients.b2 * input + self.inner.coefficients.a2 * stage_1;
        let output = self.coefficients2.b3 * stage_1 + self.inner.state_2;
        self.inner.state_2 =
            self.coefficients2.b4 * stage_1 + self.coefficients2.a4 * output + self.inner.state_3;
        self.inner.state_3 = self.coefficients2.b5 * stage_1 + self.coefficients2.a5 * output;
        Sample::from(output)
    }
}
impl BiQuadFilterLowPass24dbChannel {
    fn update_coefficients(
        &mut self,
        sample_rate: SampleRate,
        cutoff: FrequencyHz,
        passband_ripple: ParameterType,
    ) {
        let k = (PI * cutoff.0 / sample_rate.0 as f64).tan();
        let sg = passband_ripple.sinh();
        let cg = passband_ripple.cosh() * passband_ripple.cosh();

        let c0 = 1.0 / (cg - 0.853_553_390_593_273_7);
        let c1 = k * c0 * sg * 1.847_759_065_022_573_5;
        let c2 = 1.0 / (cg - 0.146_446_609_406_726_24);
        let c3 = k * c2 * sg * 0.765_366_864_730_179_6;
        let k = k * k;

        let a0 = 1.0 / (c1 + k + c0);
        let a1 = 2.0 * (c0 - k) * a0;
        let a2 = (c1 - k - c0) * a0;
        let b0 = a0 * k;
        let b1 = 2.0 * b0;
        let b2 = b0;
        self.inner.set_coefficients(CoefficientSet {
            a0,
            a1,
            a2,
            b0,
            b1,
            b2,
        });

        let a3 = 1.0 / (c3 + k + c2);
        let a4 = 2.0 * (c2 - k) * a3;
        let a5 = (c3 - k - c2) * a3;
        let b3 = a3 * k;
        let b4 = 2.0 * b3;
        let b5 = b3;
        self.coefficients2 = CoefficientSet2 { a4, a5, b3, b4, b5 };
    }
}

#[derive(Debug, Clone, Derivative, Control, Builder, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct BiQuadFilterLowPass12dbCore {
    #[control]
    #[derivative(Default(value = "500.0.into()"))]
    cutoff: FrequencyHz,
    #[control]
    q: ParameterType,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: BiQuadFilterLowPass12dbCoreEphemerals,
}
#[derive(Debug, Default, Clone)]
pub struct BiQuadFilterLowPass12dbCoreEphemerals {
    c: Configurables,
    channels: [BiQuadFilterLowPass12dbChannel; 2],
}
impl BiQuadFilterLowPass12dbCoreBuilder {
    /// The overridden Builder build() method.
    pub fn build(
        &self,
    ) -> Result<BiQuadFilterLowPass12dbCore, BiQuadFilterLowPass12dbCoreBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }
}
impl Serializable for BiQuadFilterLowPass12dbCore {}
impl Configurable for BiQuadFilterLowPass12dbCore {
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
        self.update_coefficients();
    }
}
impl TransformsAudio for BiQuadFilterLowPass12dbCore {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        match channel {
            0 | 1 => self.e.channels[channel].transform_channel(channel, input_sample),
            _ => panic!(),
        }
    }
}
impl BiQuadFilterLowPass12dbCore {
    fn update_coefficients(&mut self) {
        self.e.channels[0].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.q);
        self.e.channels[1].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.q);
    }

    pub fn cutoff(&self) -> FrequencyHz {
        self.cutoff
    }
    pub fn set_cutoff(&mut self, cutoff: FrequencyHz) {
        if self.cutoff != cutoff {
            self.cutoff = cutoff;
            self.update_coefficients();
        }
    }
    pub fn q(&self) -> ParameterType {
        self.q
    }
    pub fn set_q(&mut self, q: ParameterType) {
        if self.q != q {
            self.q = q;
            self.update_coefficients();
        }
    }
}

#[derive(Debug, Default, Clone)]
struct BiQuadFilterLowPass12dbChannel {
    inner: BiQuadFilter,
}
impl TransformsAudio for BiQuadFilterLowPass12dbChannel {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        self.inner.transform_channel(channel, input_sample)
    }
}
impl BiQuadFilterLowPass12dbChannel {
    fn update_coefficients(
        &mut self,
        sample_rate: SampleRate,
        cutoff: FrequencyHz,
        q: ParameterType,
    ) {
        let (_w0, w0cos, _w0sin, alpha) =
            BiQuadFilter::rbj_intermediates_q(sample_rate, cutoff.0, q);

        self.inner.coefficients = CoefficientSet {
            a0: 1.0 + alpha,
            a1: -2.0f64 * w0cos,
            a2: 1.0 - alpha,
            b0: (1.0 - w0cos) / 2.0f64,
            b1: (1.0 - w0cos),
            b2: (1.0 - w0cos) / 2.0f64,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Derivative, Control, Builder, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct BiQuadFilterHighPassCore {
    #[control]
    #[derivative(Default(value = "3000.0.into()"))]
    cutoff: FrequencyHz,
    #[control]
    #[derivative(Default(value = "3.0.into()"))]
    q: ParameterType,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: BiQuadFilterHighPassCoreEphemerals,
}
#[derive(Debug, Default, Clone)]
pub struct BiQuadFilterHighPassCoreEphemerals {
    c: Configurables,
    channels: [BiQuadFilterHighPassChannel; 2],
}
impl BiQuadFilterHighPassCoreBuilder {
    /// The overridden Builder build() method.
    pub fn build(&self) -> Result<BiQuadFilterHighPassCore, BiQuadFilterHighPassCoreBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }
}

impl Serializable for BiQuadFilterHighPassCore {}
impl Configurable for BiQuadFilterHighPassCore {
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
        self.update_coefficients();
    }
}
impl TransformsAudio for BiQuadFilterHighPassCore {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        match channel {
            0 | 1 => self.e.channels[channel].transform_channel(channel, input_sample),
            _ => panic!(),
        }
    }
}
impl BiQuadFilterHighPassCore {
    fn update_coefficients(&mut self) {
        self.e.channels[0].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.q);
        self.e.channels[1].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.q);
    }

    #[allow(missing_docs)]
    pub fn cutoff(&self) -> FrequencyHz {
        self.cutoff
    }

    #[allow(missing_docs)]
    pub fn set_cutoff(&mut self, cutoff: FrequencyHz) {
        if self.cutoff != cutoff {
            self.cutoff = cutoff;
            self.update_coefficients();
        }
    }

    #[allow(missing_docs)]
    pub fn q(&self) -> ParameterType {
        self.q
    }

    #[allow(missing_docs)]
    pub fn set_q(&mut self, q: ParameterType) {
        if self.q != q {
            self.q = q;
            self.update_coefficients();
        }
    }
}

#[derive(Debug, Default, Clone)]
struct BiQuadFilterHighPassChannel {
    inner: BiQuadFilter,
}
impl TransformsAudio for BiQuadFilterHighPassChannel {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        self.inner.transform_channel(channel, input_sample)
    }
}
impl BiQuadFilterHighPassChannel {
    fn update_coefficients(
        &mut self,
        sample_rate: SampleRate,
        cutoff: FrequencyHz,
        q: ParameterType,
    ) {
        let (_w0, w0cos, _w0sin, alpha) =
            BiQuadFilter::rbj_intermediates_q(sample_rate, cutoff.0, q);

        self.inner.coefficients = CoefficientSet {
            a0: 1.0 + alpha,
            a1: -2.0f64 * w0cos,
            a2: 1.0 - alpha,
            b0: (1.0 + w0cos) / 2.0f64,
            b1: -(1.0 + w0cos),
            b2: (1.0 + w0cos) / 2.0f64,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Derivative, Control, Builder, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct BiQuadFilterAllPassCore {
    #[control]
    #[derivative(Default(value = "500.0.into()"))]
    cutoff: FrequencyHz,
    #[control]
    #[derivative(Default(value = "1.0"))]
    q: ParameterType,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: BiQuadFilterAllPassCoreEphemerals,
}
#[derive(Debug, Clone, Default)]
pub struct BiQuadFilterAllPassCoreEphemerals {
    c: Configurables,
    channels: [BiQuadFilterAllPassChannel; 2],
}
impl BiQuadFilterAllPassCoreBuilder {
    /// The overridden Builder build() method.
    pub fn build(&self) -> Result<BiQuadFilterAllPassCore, BiQuadFilterAllPassCoreBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }
}
impl Serializable for BiQuadFilterAllPassCore {}
impl Configurable for BiQuadFilterAllPassCore {
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
        self.update_coefficients();
    }
}
impl TransformsAudio for BiQuadFilterAllPassCore {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        match channel {
            0 | 1 => self.e.channels[channel].transform_channel(channel, input_sample),
            _ => panic!(),
        }
    }
}
impl BiQuadFilterAllPassCore {
    fn update_coefficients(&mut self) {
        self.e.channels[0].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.q);
        self.e.channels[1].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.q);
    }

    #[allow(missing_docs)]
    pub fn cutoff(&self) -> FrequencyHz {
        self.cutoff
    }

    #[allow(missing_docs)]
    pub fn set_cutoff(&mut self, cutoff: FrequencyHz) {
        if self.cutoff != cutoff {
            self.cutoff = cutoff;
            self.update_coefficients();
        }
    }

    #[allow(missing_docs)]
    pub fn q(&self) -> ParameterType {
        self.q
    }

    #[allow(missing_docs)]
    pub fn set_q(&mut self, q: ParameterType) {
        if self.q != q {
            self.q = q;
            self.update_coefficients();
        }
    }
}
impl CanPrototype for BiQuadFilterAllPassCore {
    fn make_another(&self) -> Self {
        let mut r = Self::default();
        r.update_from_prototype(self);
        r
    }
    fn update_from_prototype(&mut self, prototype: &Self) -> &Self {
        self.set_cutoff(prototype.cutoff());
        self.set_q(prototype.q());
        self
    }
}

#[derive(Debug, Default, Clone)]
struct BiQuadFilterAllPassChannel {
    inner: BiQuadFilter,
}
impl TransformsAudio for BiQuadFilterAllPassChannel {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        self.inner.transform_channel(channel, input_sample)
    }
}
impl BiQuadFilterAllPassChannel {
    fn update_coefficients(
        &mut self,
        sample_rate: SampleRate,
        cutoff: FrequencyHz,
        q: ParameterType,
    ) {
        let (_w0, w0cos, _w0sin, alpha) =
            BiQuadFilter::rbj_intermediates_q(sample_rate, cutoff.0, q);
        self.inner.coefficients = CoefficientSet {
            a0: 1.0 + alpha,
            a1: -2.0f64 * w0cos,
            a2: 1.0 - alpha,
            b0: 1.0 - alpha,
            b1: -2.0f64 * w0cos,
            b2: 1.0 + alpha,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Derivative, Control, Builder, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct BiQuadFilterBandPassCore {
    #[control]
    #[derivative(Default(value = "3500.0.into()"))]
    cutoff: FrequencyHz,
    #[control]
    #[derivative(Default(value = "0.5.into()"))]
    bandwidth: ParameterType, // TODO: what's a good type for this concept?

    #[serde(skip)]
    #[builder(setter(skip))]
    e: BiQuadFilterBandPassCoreEphemerals,
}
#[derive(Debug, Default, Clone)]
pub struct BiQuadFilterBandPassCoreEphemerals {
    c: Configurables,
    channels: [BiQuadFilterBandPassChannel; 2],
}
impl BiQuadFilterBandPassCoreBuilder {
    /// The overridden Builder build() method.
    pub fn build(&self) -> Result<BiQuadFilterBandPassCore, BiQuadFilterBandPassCoreBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }
}
impl Serializable for BiQuadFilterBandPassCore {
    fn after_deser(&mut self) {
        self.update_coefficients()
    }
}
impl Configurable for BiQuadFilterBandPassCore {
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
        self.update_coefficients();
    }
}
impl TransformsAudio for BiQuadFilterBandPassCore {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        match channel {
            0 | 1 => self.e.channels[channel].transform_channel(channel, input_sample),
            _ => panic!(),
        }
    }
}
impl BiQuadFilterBandPassCore {
    fn update_coefficients(&mut self) {
        self.e.channels[0].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.bandwidth);
        self.e.channels[1].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.bandwidth);
    }

    #[allow(missing_docs)]
    pub fn cutoff(&self) -> FrequencyHz {
        self.cutoff
    }

    #[allow(missing_docs)]
    pub fn set_cutoff(&mut self, cutoff: FrequencyHz) {
        if self.cutoff != cutoff {
            self.cutoff = cutoff;
            self.update_coefficients();
        }
    }

    #[allow(missing_docs)]
    pub fn bandwidth(&self) -> ParameterType {
        self.bandwidth
    }

    #[allow(missing_docs)]
    pub fn set_bandwidth(&mut self, bandwidth: ParameterType) {
        if self.bandwidth != bandwidth {
            self.bandwidth = bandwidth;
            self.update_coefficients();
        }
    }
}

#[derive(Debug, Default, Clone)]
struct BiQuadFilterBandPassChannel {
    inner: BiQuadFilter,
}
impl TransformsAudio for BiQuadFilterBandPassChannel {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        self.inner.transform_channel(channel, input_sample)
    }
}
impl BiQuadFilterBandPassChannel {
    fn update_coefficients(
        &mut self,
        sample_rate: SampleRate,
        cutoff: FrequencyHz,
        bandwidth: ParameterType,
    ) {
        let (_w0, w0cos, _w0sin, alpha) =
            BiQuadFilter::rbj_intermediates_bandwidth(sample_rate, cutoff.0, bandwidth);
        self.inner.coefficients = CoefficientSet {
            a0: 1.0 + alpha,
            a1: -2.0f64 * w0cos,
            a2: 1.0 - alpha,
            b0: alpha,
            b1: 0.0,
            b2: -alpha,
        };
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Derivative, Control, Builder, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct BiQuadFilterBandStopCore {
    #[control]
    cutoff: FrequencyHz,
    #[control]
    bandwidth: ParameterType, // TODO: maybe this should be FrequencyHz

    #[serde(skip)]
    #[builder(setter(skip))]
    e: BiQuadFilterBandStopCoreEphemerals,
}
#[derive(Debug, Clone, Default)]
pub struct BiQuadFilterBandStopCoreEphemerals {
    c: Configurables,
    channels: [BiQuadFilterBandStopChannel; 2],
}
impl BiQuadFilterBandStopCoreBuilder {
    /// The overridden Builder build() method.
    pub fn build(&self) -> Result<BiQuadFilterBandStopCore, BiQuadFilterBandStopCoreBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }
}

impl Serializable for BiQuadFilterBandStopCore {}
impl Configurable for BiQuadFilterBandStopCore {
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
        self.update_coefficients();
    }
}
impl TransformsAudio for BiQuadFilterBandStopCore {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        match channel {
            0 | 1 => self.e.channels[channel].transform_channel(channel, input_sample),
            _ => panic!(),
        }
    }
}
impl BiQuadFilterBandStopCore {
    fn update_coefficients(&mut self) {
        self.e.channels[0].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.bandwidth);
        self.e.channels[1].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.bandwidth);
    }

    #[allow(missing_docs)]
    pub fn cutoff(&self) -> FrequencyHz {
        self.cutoff
    }

    #[allow(missing_docs)]
    pub fn set_cutoff(&mut self, cutoff: FrequencyHz) {
        if self.cutoff != cutoff {
            self.cutoff = cutoff;
            self.update_coefficients();
        }
    }

    #[allow(missing_docs)]
    pub fn bandwidth(&self) -> ParameterType {
        self.bandwidth
    }

    #[allow(missing_docs)]
    pub fn set_bandwidth(&mut self, bandwidth: ParameterType) {
        if self.bandwidth != bandwidth {
            self.bandwidth = bandwidth;
            self.update_coefficients();
        }
    }
}

#[derive(Debug, Default, Clone)]
struct BiQuadFilterBandStopChannel {
    inner: BiQuadFilter,
}
impl TransformsAudio for BiQuadFilterBandStopChannel {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        self.inner.transform_channel(channel, input_sample)
    }
}
impl BiQuadFilterBandStopChannel {
    fn update_coefficients(
        &mut self,
        sample_rate: SampleRate,
        cutoff: FrequencyHz,
        bandwidth: ParameterType,
    ) {
        let (_w0, w0cos, _w0sin, alpha) =
            BiQuadFilter::rbj_intermediates_bandwidth(sample_rate, cutoff.0, bandwidth);

        self.inner.coefficients = CoefficientSet {
            a0: 1.0 + alpha,
            a1: -2.0f64 * w0cos,
            a2: 1.0 - alpha,
            b0: 1.0,
            b1: -2.0f64 * w0cos,
            b2: 1.0,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Derivative, Control, Builder, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct BiQuadFilterPeakingEqCore {
    #[control]
    #[derivative(Default(value = "500.0.into()"))]
    cutoff: FrequencyHz,

    // I didn't know what to call this. RBJ says "...except for peakingEQ in
    // which A*Q is the classic EE Q." I think Q is close enough to get the gist.
    #[control]
    q: ParameterType,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: BiQuadFilterPeakingEqCoreEphemerals,
}
#[derive(Debug, Clone, Default)]
pub struct BiQuadFilterPeakingEqCoreEphemerals {
    c: Configurables,
    channels: [BiQuadFilterPeakingEqChannel; 2],
}
impl Serializable for BiQuadFilterPeakingEqCore {}
impl Configurable for BiQuadFilterPeakingEqCore {
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
        self.update_coefficients();
    }
}
impl TransformsAudio for BiQuadFilterPeakingEqCore {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        match channel {
            0 | 1 => self.e.channels[channel].transform_channel(channel, input_sample),
            _ => panic!(),
        }
    }
}
impl BiQuadFilterPeakingEqCore {
    fn update_coefficients(&mut self) {
        self.e.channels[0].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.q);
        self.e.channels[1].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.q);
    }

    #[allow(missing_docs)]
    pub fn cutoff(&self) -> FrequencyHz {
        self.cutoff
    }

    #[allow(missing_docs)]
    pub fn set_cutoff(&mut self, cutoff: FrequencyHz) {
        if self.cutoff != cutoff {
            self.cutoff = cutoff;
            self.update_coefficients();
        }
    }

    #[allow(missing_docs)]
    pub fn q(&self) -> ParameterType {
        self.q
    }

    #[allow(missing_docs)]
    pub fn set_q(&mut self, q: ParameterType) {
        if self.q != q {
            self.q = q;
            self.update_coefficients();
        }
    }
}

#[derive(Debug, Default, Clone)]
struct BiQuadFilterPeakingEqChannel {
    inner: BiQuadFilter,
}
impl TransformsAudio for BiQuadFilterPeakingEqChannel {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        self.inner.transform_channel(channel, input_sample)
    }
}
impl BiQuadFilterPeakingEqChannel {
    fn update_coefficients(
        &mut self,
        sample_rate: SampleRate,
        cutoff: FrequencyHz,
        q: ParameterType,
    ) {
        let (_w0, w0cos, _w0sin, alpha) = BiQuadFilter::rbj_intermediates_q(
            sample_rate,
            cutoff.0,
            std::f64::consts::FRAC_1_SQRT_2,
        );
        let a = 10f64.powf(q / 10.0f64).sqrt();

        self.inner.coefficients = CoefficientSet {
            a0: 1.0 + alpha / a,
            a1: -2.0f64 * w0cos,
            a2: 1.0 - alpha / a,
            b0: 1.0 + alpha * a,
            b1: -2.0f64 * w0cos,
            b2: 1.0 - alpha * a,
        }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Derivative, Control, Builder, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct BiQuadFilterLowShelfCore {
    #[control]
    #[derivative(Default(value = "500.0.into()"))]
    cutoff: FrequencyHz,
    #[control]
    db_gain: ParameterType,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: BiQuadFilterLowShelfCoreEphemerals,
}
#[derive(Debug, Clone, Default)]
pub struct BiQuadFilterLowShelfCoreEphemerals {
    c: Configurables,
    channels: [BiQuadFilterLowShelfChannel; 2],
}

impl Serializable for BiQuadFilterLowShelfCore {}
impl Configurable for BiQuadFilterLowShelfCore {
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
        self.update_coefficients();
    }
}
impl TransformsAudio for BiQuadFilterLowShelfCore {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        match channel {
            0 | 1 => self.e.channels[channel].transform_channel(channel, input_sample),
            _ => panic!(),
        }
    }
}
#[allow(missing_docs)]
impl BiQuadFilterLowShelfCore {
    fn update_coefficients(&mut self) {
        self.e.channels[0].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.db_gain);
        self.e.channels[1].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.db_gain);
    }

    pub fn cutoff(&self) -> FrequencyHz {
        self.cutoff
    }
    pub fn set_cutoff(&mut self, cutoff: FrequencyHz) {
        if self.cutoff != cutoff {
            self.cutoff = cutoff;
            self.update_coefficients();
        }
    }
    pub fn db_gain(&self) -> ParameterType {
        self.db_gain
    }
    pub fn set_db_gain(&mut self, db_gain: ParameterType) {
        if self.db_gain != db_gain {
            self.db_gain = db_gain;
            self.update_coefficients();
        }
    }
}

#[derive(Debug, Default, Clone)]
struct BiQuadFilterLowShelfChannel {
    inner: BiQuadFilter,
}
impl TransformsAudio for BiQuadFilterLowShelfChannel {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        self.inner.transform_channel(channel, input_sample)
    }
}
impl BiQuadFilterLowShelfChannel {
    fn update_coefficients(
        &mut self,
        sample_rate: SampleRate,
        cutoff: FrequencyHz,
        db_gain: ParameterType,
    ) {
        let a = 10f64.powf(db_gain / 10.0f64).sqrt();
        let (_w0, w0cos, _w0sin, alpha) =
            BiQuadFilter::rbj_intermediates_shelving(sample_rate, cutoff.0, a, 1.0);

        self.inner.coefficients = CoefficientSet {
            a0: (a + 1.0) + (a - 1.0) * w0cos + 2.0 * a.sqrt() * alpha,
            a1: -2.0 * ((a - 1.0) + (a + 1.0) * w0cos),
            a2: (a + 1.0) + (a - 1.0) * w0cos - 2.0 * a.sqrt() * alpha,
            b0: a * ((a + 1.0) - (a - 1.0) * w0cos + 2.0 * a.sqrt() * alpha),
            b1: 2.0 * a * ((a - 1.0) - (a + 1.0) * w0cos),
            b2: a * ((a + 1.0) - (a - 1.0) * w0cos - 2.0 * a.sqrt() * alpha),
        };
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Derivative, Control, Builder, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct BiQuadFilterHighShelfCore {
    #[control]
    #[derivative(Default(value = "500.0.into()"))]
    cutoff: FrequencyHz,
    #[control]
    db_gain: ParameterType,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: BiQuadFilterHighShelfCoreEphemerals,
}
#[derive(Debug, Clone, Default)]
pub struct BiQuadFilterHighShelfCoreEphemerals {
    c: Configurables,
    channels: [BiQuadFilterHighShelfChannel; 2],
}

impl Serializable for BiQuadFilterHighShelfCore {}
impl Configurable for BiQuadFilterHighShelfCore {
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
        self.update_coefficients();
    }
}
impl TransformsAudio for BiQuadFilterHighShelfCore {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        match channel {
            0 | 1 => self.e.channels[channel].transform_channel(channel, input_sample),
            _ => panic!(),
        }
    }
}
impl BiQuadFilterHighShelfCore {
    fn update_coefficients(&mut self) {
        self.e.channels[0].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.db_gain);
        self.e.channels[1].update_coefficients(self.e.c.sample_rate(), self.cutoff, self.db_gain);
    }

    pub fn cutoff(&self) -> FrequencyHz {
        self.cutoff
    }
    pub fn set_cutoff(&mut self, cutoff: FrequencyHz) {
        if self.cutoff != cutoff {
            self.cutoff = cutoff;
            self.update_coefficients();
        }
    }
    pub fn db_gain(&self) -> ParameterType {
        self.db_gain
    }
    pub fn set_db_gain(&mut self, db_gain: ParameterType) {
        if self.db_gain != db_gain {
            self.db_gain = db_gain;
            self.update_coefficients();
        }
    }
}

#[derive(Debug, Default, Clone)]
struct BiQuadFilterHighShelfChannel {
    inner: BiQuadFilter,
}
impl TransformsAudio for BiQuadFilterHighShelfChannel {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        self.inner.transform_channel(channel, input_sample)
    }
}
impl BiQuadFilterHighShelfChannel {
    fn update_coefficients(
        &mut self,
        sample_rate: SampleRate,
        cutoff: FrequencyHz,
        db_gain: ParameterType,
    ) {
        let a = 10f64.powf(db_gain / 10.0f64).sqrt();
        let (_w0, w0cos, _w0sin, alpha) =
            BiQuadFilter::rbj_intermediates_shelving(sample_rate, cutoff.0, a, 1.0);

        self.inner.coefficients = CoefficientSet {
            a0: (a + 1.0) - (a - 1.0) * w0cos + 2.0 * a.sqrt() * alpha,
            a1: 2.0 * ((a - 1.0) - (a + 1.0) * w0cos),
            a2: (a + 1.0) - (a - 1.0) * w0cos - 2.0 * a.sqrt() * alpha,
            b0: a * ((a + 1.0) + (a - 1.0) * w0cos + 2.0 * a.sqrt() * alpha),
            b1: -2.0 * a * ((a - 1.0) + (a + 1.0) * w0cos),
            b2: a * ((a + 1.0) + (a - 1.0) * w0cos - 2.0 * a.sqrt() * alpha),
        }
    }
}

/// This filter does nothing, expensively. It exists for debugging. I might
/// delete it later.
#[derive(Debug, Clone, Derivative, Control, Builder, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct BiQuadFilterNoneCore {
    #[serde(skip)]
    #[builder(setter(skip))]
    e: BiQuadFilterNoneCoreEphemerals,
}
#[derive(Debug, Clone, Default)]
pub struct BiQuadFilterNoneCoreEphemerals {
    c: Configurables,
    channels: [BiQuadFilter; 2],
}

impl Serializable for BiQuadFilterNoneCore {}
impl Configurable for BiQuadFilterNoneCore {
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
impl TransformsAudio for BiQuadFilterNoneCore {
    fn transform_channel(&mut self, channel: usize, input_sample: Sample) -> Sample {
        match channel {
            0 | 1 => self.e.channels[channel].transform_channel(channel, input_sample),
            _ => panic!(),
        }
    }
}

#[derive(Clone, Debug)]
struct CoefficientSet {
    a0: f64,
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
}
impl Default for CoefficientSet {
    // This is an identity set.
    fn default() -> Self {
        Self {
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct CoefficientSet2 {
    // a3 isn't needed right now
    a4: f64,
    a5: f64,
    b3: f64,
    b4: f64,
    b5: f64,
}

/// <https://en.wikipedia.org/wiki/Digital_biquad_filter>
#[derive(Clone, Debug, Default)]
pub struct BiQuadFilter {
    coefficients: CoefficientSet,

    // Working variables
    sample_m1: f64, // "sample minus two" or x(n-2)
    sample_m2: f64,
    output_m1: f64,
    output_m2: f64,

    state_0: f64,
    state_1: f64,
    state_2: f64,
    state_3: f64,
}
impl TransformsAudio for BiQuadFilter {
    // Everyone but LowPassFilter24db uses this implementation
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        let s64 = input_sample.0;
        let r = (self.coefficients.b0 / self.coefficients.a0) * s64
            + (self.coefficients.b1 / self.coefficients.a0) * self.sample_m1
            + (self.coefficients.b2 / self.coefficients.a0) * self.sample_m2
            - (self.coefficients.a1 / self.coefficients.a0) * self.output_m1
            - (self.coefficients.a2 / self.coefficients.a0) * self.output_m2;

        // Scroll everything forward in time.
        self.sample_m2 = self.sample_m1;
        self.sample_m1 = s64;

        self.output_m2 = self.output_m1;
        self.output_m1 = r;
        Sample::from(r)
    }
}
impl BiQuadFilter {
    // A placeholder for an intelligent mapping of 0.0..=1.0 to a reasonable Q
    // range
    #[allow(dead_code)]
    pub fn denormalize_q(value: Normal) -> ParameterType {
        value.0 * value.0 * 10.0 + 0.707
    }

    // A placeholder for an intelligent mapping of 0.0..=1.0 to a reasonable
    // 24db passband parameter range
    #[allow(dead_code)]
    pub fn convert_passband(value: f32) -> f32 {
        value * 100.0 + 0.1
    }

    // Excerpted from Robert Bristow-Johnson's audio cookbook to explain various
    // parameters
    //
    // Fs (the sampling frequency)
    //
    // f0 ("wherever it's happenin', man."  Center Frequency or Corner
    //     Frequency, or shelf midpoint frequency, depending on which filter
    //     type.  The "significant frequency".)
    //
    // dBgain (used only for peaking and shelving filters)
    //
    // Q (the EE kind of definition, except for peakingEQ in which A*Q is the
    // classic EE Q.  That adjustment in definition was made so that a boost of
    // N dB followed by a cut of N dB for identical Q and f0/Fs results in a
    // precisely flat unity gain filter or "wire".)
    //
    // - _or_ BW, the bandwidth in octaves (between -3 dB frequencies for BPF
    //     and notch or between midpoint (dBgain/2) gain frequencies for peaking
    //     EQ)
    //
    // - _or_ S, a "shelf slope" parameter (for shelving EQ only).  When S = 1,
    //     the shelf slope is as steep as it can be and remain monotonically
    //     increasing or decreasing gain with frequency.  The shelf slope, in
    //     dB/octave, remains proportional to S for all other values for a fixed
    //     f0/Fs and dBgain.

    fn rbj_intermediates_q(
        sample_rate: SampleRate,
        cutoff: ParameterType,
        q: ParameterType,
    ) -> (f64, f64, f64, f64) {
        let w0 = 2.0f64 * PI * cutoff / sample_rate.0 as f64;
        let w0cos = w0.cos();
        let w0sin = w0.sin();
        let alpha = w0sin / (2.0 * q.max(ParameterType::EPSILON));
        (w0, w0cos, w0sin, alpha)
    }

    fn rbj_intermediates_bandwidth(
        sample_rate: SampleRate,
        cutoff: ParameterType,
        bandwidth: ParameterType,
    ) -> (f64, f64, f64, f64) {
        let w0 = 2.0f64 * PI * cutoff / sample_rate.0 as f64;
        let w0cos = w0.cos();
        let w0sin = w0.sin();
        let alpha = w0sin
            * (2.0f64.ln() / 2.0 * bandwidth.max(ParameterType::EPSILON) * w0 / w0.sin()).sinh();
        (w0, w0cos, w0sin, alpha)
    }

    fn rbj_intermediates_shelving(
        sample_rate: SampleRate,
        cutoff: ParameterType,
        db_gain: ParameterType,
        s: f64,
    ) -> (f64, f64, f64, f64) {
        let w0 = 2.0f64 * PI * cutoff / sample_rate.0 as f64;
        let w0cos = w0.cos();
        let w0sin = w0.sin();
        let alpha = w0sin / 2.0
            * ((db_gain + 1.0 / db_gain.max(ParameterType::EPSILON)) * (1.0 / s - 1.0) + 2.0)
                .sqrt();
        (w0, w0cos, w0sin, alpha)
    }

    fn set_coefficients(&mut self, coefficient_set: CoefficientSet) {
        self.coefficients = coefficient_set;
    }
}

#[cfg(test)]
mod tests {
    // TODO: get FFT working, and then write tests.
}
