// Copyright (c) 2024 Mike Tsao

//! Support for project organization and rendering.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{BasicProject, Projects, TrackUid, TrackUidFactory};
}

pub use project::BasicProject;
pub use track::{TrackTitle, TrackUid, TrackUidFactory};
pub use traits::Projects;

mod project;
mod track;
mod traits;
