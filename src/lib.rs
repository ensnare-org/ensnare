// Copyright (c) 2024 Mike Tsao

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs, unused_imports, unused_variables)]
#![allow(dead_code)] // TODO: remove when big migration is complete
#![allow(rustdoc::private_intra_doc_links)]

//! Ensnare helps create digital audio.

#[cfg(feature = "std")]
pub use version::app_version;

/// A collection of imports that are useful to users of this crate. `use
/// ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    pub use super::{
        automation::prelude::*, elements::prelude::*, entities::prelude::*,
        orchestration::prelude::*, traits::prelude::*, types::prelude::*, util::prelude::*,
    };
}

pub mod automation;
pub mod cores;
pub mod elements;
pub mod entities;
pub mod orchestration;
pub mod traits;
pub mod types;
pub mod util;

#[cfg(feature = "std")]
mod version;
