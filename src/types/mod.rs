// Copyright (c) 2024 Mike Tsao

//! Common data types used throughout the system.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        u4, u7, BipolarNormal, FrequencyHz, MidiChannel, MusicalTime, Normal, ParameterType, Ratio,
        Sample, SampleRate, StereoSample, Tempo, TimeRange, TimeSignature, Uid, UidFactory,
    };
}

pub use {
    channels::{BoundedCrossbeamChannel, CrossbeamChannel},
    midi::{u4, u7, MidiChannel, MidiPortDescriptor},
    numbers::{FrequencyHz, ParameterType, Ratio, Sample, StereoSample},
    ranges::{BipolarNormal, Normal},
    time::{MusicalTime, SampleRate, Seconds, Tempo, TimeRange, TimeSignature},
    uid::{IsUid, Uid, UidFactory},
};

mod channels;
mod midi;
mod numbers;
mod ranges;
mod time;
mod uid;
