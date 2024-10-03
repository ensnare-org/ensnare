// Copyright (c) 2024 Mike Tsao

//! Provides test instruments, controllers, and effects. Test entities are designed with
/// an emphasis on instrumentation and introspection, rather than useful audio
/// functionality.
///
#[cfg(test)]
pub use {
    controllers::{TestController, TestControllerAlwaysSendsMidiMessage, TestControllerTimed},
    effects::{TestEffect, TestEffectNegatesInput},
    factory::TestEntities,
    instruments::{TestAudioSource, TestInstrument, TestInstrumentCountsMidiMessages},
};

mod controllers;
mod effects;
mod factory;
mod instruments;
