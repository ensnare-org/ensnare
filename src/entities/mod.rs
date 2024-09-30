// Copyright (c) 2024 Mike Tsao

//! Built-in musical instruments and supporting infrastructure.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{register_simple_entities, SimpleController, SimpleEffect, SimpleInstrument};

    // #[cfg(test)]
    // TODO: when ensnare-v1 crate is gone and the consumers of this are in this crate, restore this directive
    pub use super::{
        register_test_entities, TestAudioSource, TestController,
        TestControllerAlwaysSendsMidiMessage, TestControllerTimed, TestEffectNegatesInput,
        TestInstrument, TestInstrumentCountsMidiMessages,
    };

    pub use super::{BuiltInEntities, EntityFactory, EntityKey, EntityUidFactory, Timer};
}

pub use built_in::*;
pub use infra::{EntityFactory, EntityKey, EntityUidFactory};
pub use simple::*;

// #[cfg(test)]
// TODO: when ensnare-v1 crate is gone and the consumers of this are in this crate, restore this directive
pub use test_entities::*;

mod built_in;
mod infra;
#[cfg(not(feature = "simple"))]
mod simple;
// #[cfg(test)]
// TODO: when ensnare-v1 crate is gone and the consumers of this are in this crate, restore this directive
mod test_entities;
