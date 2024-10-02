// Copyright (c) 2024 Mike Tsao

use crate::types::MidiNote;
use core::ops::RangeInclusive;

/// A wrapper around `RangeInclusive<MidiNote>`.
#[derive(Clone)]
pub struct MidiNoteRange(pub RangeInclusive<MidiNote>);
impl Default for MidiNoteRange {
    fn default() -> Self {
        Self(MidiNote::MIN..=MidiNote::MAX)
    }
}
impl MidiNoteRange {
    #[allow(missing_docs)]
    pub fn start(&self) -> MidiNote {
        *self.0.start()
    }

    #[allow(missing_docs)]
    pub fn end(&self) -> MidiNote {
        *self.0.end()
    }
}
