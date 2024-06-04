// Copyright (c) 2024 Mike Tsao

//! Wrappers around third-party crates that make them easier to use with
//! crossbeam channels.

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

/// The most commonly used imports.
pub mod prelude {
    #[cfg(feature = "audio")]
    pub use super::{CpalAudioService, CpalAudioServiceEvent, CpalAudioServiceInput};
    #[cfg(feature = "midi")]
    pub use super::{MidiService, MidiServiceEvent, MidiServiceInput};
}

#[cfg(feature = "audio")]
pub use audio::{
    AudioSampleType, AudioStereoSampleType, CpalAudioService, CpalAudioServiceEvent,
    CpalAudioServiceInput,
};
#[cfg(feature = "midi")]
pub use midi::{MidiService, MidiServiceEvent, MidiServiceInput};

#[cfg(feature = "audio")]
mod audio;
#[cfg(feature = "midi")]
mod midi;
