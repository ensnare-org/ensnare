// Copyright (c) 2024 Mike Tsao

//! System utilities.

/// Commonly used imports.
pub mod prelude {
    pub use super::{init_sample_libraries, rng::Rng, Paths};
}

#[cfg(feature = "std")]
pub use mod_serial::ModSerial;
pub use {
    library::{
        init_sample_libraries, Kit, KitIndex, KitItem, KitLibrary, SampleIndex, SampleLibrary,
        SampleSource,
    },
    midi::{MidiNoteMinder, MidiUtils},
    paths::{FileType, PathType, Paths},
    rng::Rng,
};

mod library;
mod midi;
mod paths;
mod rng;

#[cfg(feature = "std")]
mod mod_serial;
