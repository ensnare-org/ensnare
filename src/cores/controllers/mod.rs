// Copyright (c) 2024 Mike Tsao

//! Controllers are musical devices that emit control events rather than audio.
//! A good example is an arpeggiator, which produces MIDI messages.

pub use arpeggiator::{ArpeggiatorCore, ArpeggiatorCoreBuilder, ArpeggioMode};
pub use lfo::{LfoControllerCore, LfoControllerCoreBuilder};
pub use passthrough::{SignalPassthroughControllerCore, SignalPassthroughControllerCoreBuilder};
pub use timer::{TimerCore, TimerCoreBuilder};
pub use trigger::{TriggerCore, TriggerCoreBuilder};

mod arpeggiator;
mod lfo;
mod passthrough;
mod timer;
mod trigger;
