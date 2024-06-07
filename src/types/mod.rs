// Copyright (c) 2024 Mike Tsao

//! Common data types used throughout the system.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        u4, u7, BipolarNormal, CrossbeamChannel, FrequencyHz, MidiChannel, MidiMessage,
        MusicalTime, Normal, ParameterType, Ratio, Sample, SampleRate, SampleType, Seconds,
        StereoSample, Tempo, TimeRange, TimeSignature, Uid, UidFactory, ViewRange,
    };
}

pub use {
    channels::{BoundedCrossbeamChannel, CrossbeamChannel},
    general_midi::{GeneralMidiPercussionCode, GeneralMidiProgram},
    midi::{u4, u7, MidiChannel, MidiMessage, MidiPortDescriptor},
    note::MidiNote,
    numbers::{FrequencyHz, ParameterType, Ratio, Sample, SampleType, StereoSample},
    ranges::{BipolarNormal, Normal},
    time::{MusicalTime, SampleRate, Seconds, Tempo, TimeRange, TimeSignature, ViewRange},
    uid::{IsUid, Uid, UidFactory},
};

mod channels;
mod general_midi;
mod midi;
mod note;
mod numbers;
mod ranges;
mod time;
mod uid;
