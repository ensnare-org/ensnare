// Copyright (c) 2024 Mike Tsao

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs, unused_imports, unused_variables)]
#![allow(dead_code)] // TODO: remove when big migration is complete
#![allow(rustdoc::private_intra_doc_links)]

//! Ensnare creates digital audio, with a focus on music.
//!
//! There are several ways to develop a music application with Ensnare,
//! depending on the level of control you need.
//!
//! * *Easiest, but least control*: Use a [Project] to describe a musical
//! composition and arrangement, then render the song with the
//! [StereoSample](types::StereoSample) iterator obtained from
//! [BasicProject::render()](crate::traits::Projects::)
//! iterator until you have rendered the entire song.
//!
//! Another approach is to instantiate [Composer] for expressing a musical
//! composition, [Automator] for automating control events, and [Orchestrator]
//! for arranging musical instruments and effects into tracks, and then bringing
//! them together in the main loop.
//!
//! For even more control, you can create individual [entities](crate::entities)
//! and assemble them as you need.
//!
//! Finally, you can use the bare musical [cores](crate::cores) and obtain
//! digital audio samples directly from them.

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
