// Copyright (c) 2024 Mike Tsao

//! Creation and representation of music scores.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        sequencers::{PatternSequencer, PatternSequencerBuilder},
        ArrangementUid, Composer, MidiNoteRange, Note, Pattern, PatternBuilder, PatternUid,
        PatternUidFactory,
    };
}

pub use arrangement::*;
pub use composer::*;
pub use note::*;
pub use pattern::*;
pub use sequencers::*;
pub use types::*;

mod arrangement;
mod composer;
mod note;
mod pattern;
mod sequencers;
mod types;
