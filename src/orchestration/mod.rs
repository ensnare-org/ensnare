// Copyright (c) 2024 Mike Tsao

//! Support for project organization and rendering.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{Project, TrackUid, TrackUidFactory};
}

pub use project::Project;
pub use track::{TrackTitle, TrackUid, TrackUidFactory};

mod project;
mod track;
