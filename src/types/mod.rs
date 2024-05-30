// Copyright (c) 2024 Mike Tsao

//! Common data types used throughout the system.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        BipolarNormal, FrequencyHz, MusicalTime, Normal, ParameterType, Ratio, Sample, SampleRate,
        StereoSample, Tempo, TimeRange, TimeSignature, Uid, UidFactory,
    };
}

pub use {
    numbers::{FrequencyHz, ParameterType, Ratio, Sample, StereoSample},
    ranges::{BipolarNormal, Normal},
    time::{MusicalTime, SampleRate, Seconds, Tempo, TimeRange, TimeSignature},
    uid::{IsUid, Uid, UidFactory},
};

mod midi;
mod numbers;
mod ranges;
mod time;
mod uid;
