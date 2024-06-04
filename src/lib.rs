// Copyright (c) 2024 Mike Tsao

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! Ensnare helps create digital audio.

#[cfg(feature = "std")]
pub use version::app_version;

/// A collection of imports that are useful to users of this crate. `use
/// ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    pub use super::{
        automation::prelude::*, entities::prelude::*, orchestration::prelude::*,
        traits::prelude::*, types::prelude::*, util::prelude::*,
    };
}

pub mod automation;
pub mod entities;
pub mod orchestration;
pub mod traits;
pub mod types;
pub mod util;

#[cfg(feature = "std")]
mod version;
