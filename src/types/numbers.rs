// Copyright (c) 2024 Mike Tsao

//! Numeric types used throughout the system.

use core::ops::Add;

/// The primitive Rust type of a single audio sample.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct SampleType(f64);
impl Add<Self> for SampleType {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

/// [Sample] represents a single-channel audio sample.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Sample(pub SampleType);
impl Sample {
    /// The [SampleType] value of silence.
    pub const SILENCE_VALUE: SampleType = SampleType(0.0);
    /// A [Sample] that is silent.
    pub const SILENCE: Sample = Sample(Self::SILENCE_VALUE);
    /// The maximum positive [SampleType] value of silence.
    pub const MAX_VALUE: SampleType = SampleType(1.0);
    /// A [Sample] having the maximum positive value.
    pub const MAX: Sample = Sample(Self::MAX_VALUE);
    /// The maximum negative [SampleType] value.
    pub const MIN_VALUE: SampleType = SampleType(-1.0);
    /// A [Sample] having the maximum negative value.
    pub const MIN: Sample = Sample(Self::MIN_VALUE);
}
// I predict this conversion will someday be declared evil. We're naively
// averaging the two channels. I'm not sure this makes sense in all situations.
impl From<StereoSample> for Sample {
    fn from(value: StereoSample) -> Self {
        Sample::from((value.0 .0 .0 + value.1 .0 .0) / 2.0)
    }
}
impl From<SampleType> for Sample {
    fn from(value: SampleType) -> Self {
        Sample(value)
    }
}
impl From<f64> for Sample {
    fn from(value: f64) -> Self {
        Sample(SampleType(value))
    }
}
impl From<f32> for Sample {
    fn from(value: f32) -> Self {
        Sample(SampleType(value as f64))
    }
}

/// [StereoSample] is a two-channel sample.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct StereoSample(pub Sample, pub Sample);
impl StereoSample {
    /// Silence (0.0).
    pub const SILENCE: StereoSample = StereoSample(Sample::SILENCE, Sample::SILENCE);
    /// The loudest positive value (1.0).
    pub const MAX: StereoSample = StereoSample(Sample::MAX, Sample::MAX);
    /// The loudest negative value (-1.0).
    pub const MIN: StereoSample = StereoSample(Sample::MIN, Sample::MIN);

    /// Creates a new [StereoSample] from left and right [Sample]s.
    pub fn new(left: Sample, right: Sample) -> Self {
        Self(left, right)
    }
}
impl From<Sample> for StereoSample {
    fn from(value: Sample) -> Self {
        Self(value, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_to_stereo() {
        assert_eq!(StereoSample::from(Sample::MIN), StereoSample::MIN);
        assert_eq!(StereoSample::from(Sample::SILENCE), StereoSample::SILENCE);
        assert_eq!(StereoSample::from(Sample::MAX), StereoSample::MAX);
    }

    #[test]
    fn stereo_to_mono() {
        assert_eq!(Sample::from(StereoSample::MIN), Sample::MIN);
        assert_eq!(Sample::from(StereoSample::SILENCE), Sample::SILENCE);
        assert_eq!(Sample::from(StereoSample::MAX), Sample::MAX);

        assert_eq!(
            Sample::from(StereoSample::new(1.0.into(), 0.0.into())),
            Sample::from(0.5)
        );
    }
}
