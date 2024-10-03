// Copyright (c) 2024 Mike Tsao

//! Entities and supporting infrastructure.
//!
//! An Entity is something that implements the [Entity](crate::traits::Entity)
//! trait. It's helpful to think of three Entity types:
//!
//! - *instrument*: A musical instrument that responds to MIDI events by
//!   producing sound. A MIDI synthesizer is an instrument.
//! - *controller*: A device that produces MIDI events and/or
//!   [WorkEvents](crate::traits::WorkEvent). A MIDI sequencer is a controller.
//! - *effect*: A device that modifies an audio signal. A reverb is an effect.
//!
//! These types are not strictly defined, and an Entity can be a hybrid of
//! types. For example, [Arpeggiator] is an instrument in the sense that it
//! responds to MIDI input, and it's also a controller in the sense that it
//! produces MIDI events. As another example, [SignalPassthroughController] is
//! an effect in the sense that it accepts an audio input, and it is a
//! controller because it uses that input signal to generate
//! [WorkEvents](crate::traits::WorkEvent).
//!
//! Occasionally this documentation will refer to Entities as "instruments,"
//! even though an instrument is a specific kind of Entity. This ambiguity is
//! the lesser of two evils; sometimes the term "entity" is too technical, but
//! from context it's usually clear that we mean "instrument" to include all
//! kinds of Entities. Sorry for any confusion.
//!
//! Some of the submodules contain a group of entities:
//!
//! - `built-in`: fully functional musical instruments, controllers, and effects
//!   that are included with the crate.
//! - `simple`: minimally functional examples of each kind of entity. Useful
//!   mainly for development.
//! - `test_entities`: test-focused entities that often have very specific
//!   functionality, and that usually have instrumentation and introspection
//!   capabilities, useful for test assertions, that a regular entity wouldn't
//!   have.

/// The most commonly used imports.
pub mod prelude {
    #[cfg(feature = "simple")]
    pub use super::{SimpleController, SimpleEffect, SimpleEntities, SimpleInstrument};

    #[cfg(test)]
    pub use super::{
        TestAudioSource, TestController, TestControllerAlwaysSendsMidiMessage, TestControllerTimed,
        TestEffectNegatesInput, TestEntities, TestInstrument, TestInstrumentCountsMidiMessages,
    };

    pub use super::{BuiltInEntities, EntityFactory, EntityKey, EntityUidFactory, Timer};
}

pub use built_in::*;
pub use infra::{EntityFactory, EntityKey, EntityUidFactory};

mod built_in;
mod infra;

#[cfg(feature = "simple")]
pub use simple::*;
#[cfg(feature = "simple")]
mod simple;

#[cfg(test)]
pub use test_entities::*;
#[cfg(test)]
mod test_entities;
