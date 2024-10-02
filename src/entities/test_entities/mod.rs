// Copyright (c) 2024 Mike Tsao

//! Test instruments, controllers, and effects. Test entities are designed with
/// an emphasis on instrumentation and introspection, rather than useful audio
/// functionality.
///
#[cfg(test)]
pub use {
    controllers::{TestController, TestControllerAlwaysSendsMidiMessage, TestControllerTimed},
    effects::{TestEffect, TestEffectNegatesInput},
    factory::register_test_entities,
    instruments::{TestAudioSource, TestInstrument, TestInstrumentCountsMidiMessages},
};

mod controllers;
mod effects;
mod factory;
mod instruments;
