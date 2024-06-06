// Copyright (c) 2024 Mike Tsao

//! Support for project organization and rendering.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{ProjectV2, Projects, TrackUid, TrackUidFactory};
}

pub use project::ProjectV2;
pub use track::{TrackTitle, TrackUid, TrackUidFactory};
pub use traits::Projects;

mod project;
mod track;
mod traits;
