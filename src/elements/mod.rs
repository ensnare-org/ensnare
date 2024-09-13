// Copyright (c) 2024 Mike Tsao

//! Reusable building blocks for other parts of the system, especially musical
//! instruments and effects.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{
        generators::{Envelope, EnvelopeBuilder, Oscillator, OscillatorBuilder, Waveform},
        modulators::Dca,
        synthesizers::Synthesizer,
        transport::{Transport, TransportBuilder},
        voices::{StealingVoiceStore, VoiceCount, VoicePerNoteStore, VoiceStore},
    };
}

pub use generators::{Envelope, EnvelopeBuilder, Oscillator, OscillatorBuilder, Waveform};
pub use modulators::Dca;
pub use synthesizers::Synthesizer;
pub use transport::{Transport, TransportBuilder};
pub use voices::{StealingVoiceStore, VoiceCount, VoicePerNoteStore, VoiceStore};

/// Building blocks for signal generation.
mod generators;
/// Building blocks for signal modulation.
mod modulators;
/// Scaffolding for building synthesizers.
mod synthesizers;
/// Management of time.
mod transport;
/// Scaffolding for managing multiple voices.
mod voices;
