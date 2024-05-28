// Copyright (c) 2024 Mike Tsao

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! Ensnare helps create digital audio.

pub use version::app_version;

/// A collection of imports that are useful to users of this crate. `use
/// ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    pub use super::types::prelude::*;
}

pub mod types;

mod version;
