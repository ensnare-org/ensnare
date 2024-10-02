// Copyright (c) 2024 Mike Tsao

//! Support for project organization and rendering.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{BasicProject, Project, ProjectTitle, Projects, TrackUid, TrackUidFactory};
}

pub use {
    basic_project::{BasicProject, SignalChainItem, TrackInfo, TrackViewMode},
    humidity::Humidifier,
    orchestrator::Orchestrator,
    project::{AudioSenderFn, Project, ProjectTitle, ProjectViewState},
    repositories::{EntityRepository, TrackRepository},
    track::{TrackTitle, TrackUid, TrackUidFactory},
    traits::Projects,
};

use {bus::BusStation, midi_router::MidiRouter};

mod basic_project;
mod bus;
mod humidity;
mod midi_router;
mod orchestrator;
mod project;
mod repositories;
mod track;
mod traits;
