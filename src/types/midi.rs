// Copyright (c) 2024 Mike Tsao

use serde::{Deserialize, Serialize};
use synonym::Synonym;

pub use midly::num::{u4, u7};

/// Newtype for MIDI channel.
#[derive(Synonym, Serialize, Deserialize)]
pub struct MidiChannel(pub u8);
#[allow(missing_docs)]
impl MidiChannel {
    pub const MIN_VALUE: u8 = 0;
    pub const MAX_VALUE: u8 = 15; // inclusive
    pub const DRUM_VALUE: u8 = 10;
    pub const DRUM: Self = Self(Self::DRUM_VALUE);

    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}
impl From<u4> for MidiChannel {
    fn from(value: u4) -> Self {
        Self(value.as_int())
    }
}

/// Provides user-friendly strings for displaying available MIDI ports.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MidiPortDescriptor {
    /// The port descriptor's index.
    pub index: usize,
    /// The port descriptor's human-readable name.
    pub name: String,
}
impl std::fmt::Display for MidiPortDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}
