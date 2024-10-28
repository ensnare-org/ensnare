// Copyright (c) 2024 Mike Tsao

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs, unused_imports, unused_variables)]
#![allow(dead_code)] // TODO: remove when big migration is complete
#![allow(rustdoc::private_intra_doc_links)]

//! Ensnare makes it easier to create digital audio applications, particularly
//! applications that focus on music.

/// A collection of imports that are useful to users of this crate. `use
/// ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    pub use super::{
        automation::prelude::*, composition::prelude::*, elements::prelude::*,
        entities::prelude::*, orchestration::prelude::*, traits::prelude::*, types::prelude::*,
        util::prelude::*,
    };
}

// Fundamental structures that are important enough to re-export at top level.
#[cfg(feature = "std")]
pub use version::app_version;
pub use {
    automation::Automator,
    composition::Composer,
    orchestration::{BasicProject, Orchestrator, Project},
};

pub mod automation;
pub mod composition;
pub mod cores;
#[cfg(feature = "egui")]
pub mod egui;
pub mod elements;
pub mod entities;
pub mod orchestration;
pub mod traits;
pub mod types;
pub mod util;

#[cfg(feature = "std")]
mod version;
