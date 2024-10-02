// Copyright (c) 2024 Mike Tsao

//! Test instruments, controllers, and effects. Test entities are designed with
/// an emphasis on instrumentation and introspection, rather than useful audio
/// functionality.
///
pub use controllers::{TestController, TestControllerAlwaysSendsMidiMessage, TestControllerTimed};
pub use effects::{TestEffect, TestEffectNegatesInput};
pub use factory::register_test_entities;
pub use instruments::{TestAudioSource, TestInstrument, TestInstrumentCountsMidiMessages};

mod controllers;
mod effects;
mod factory;
mod instruments;
