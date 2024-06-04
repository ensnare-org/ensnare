// Copyright (c) 2024 Mike Tsao

//! System utilities.

/// Commonly used imports.
pub mod prelude {
    pub use super::rng::Rng;
}

pub use midi::MidiUtils;
pub use rng::Rng;

mod midi;
mod rng;
