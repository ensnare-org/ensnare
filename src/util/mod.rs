// Copyright (c) 2024 Mike Tsao

//! System utilities.

/// Commonly used imports.
pub mod prelude {
    pub use super::{channels::CrossbeamChannel, rng::Rng};
}

pub use channels::CrossbeamChannel;
pub use rng::Rng;

mod channels;
mod rng;
