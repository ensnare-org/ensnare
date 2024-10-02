// Copyright (c) 2024 Mike Tsao

pub use {fm::FmSynth, subtractive::SubtractiveSynth};

mod fm;
mod subtractive;

#[cfg(feature = "hound")]
pub use {drumkit::Drumkit, sampler::Sampler};

#[cfg(feature = "hound")]
mod drumkit;
#[cfg(feature = "hound")]
mod sampler;
