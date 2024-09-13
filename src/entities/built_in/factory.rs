// Copyright (c) 2024 Mike Tsao

use super::*;
use crate::{
    cores::{DelayCoreBuilder, GainCoreBuilder, ReverbCoreBuilder, TimerCoreBuilder},
    prelude::*,
};

/// A collection of all entities that are suitable for normal use. Allows the
/// creation of an [EntityFactory] that lets apps refer to entities by
/// [EntityKey] rather than having to import and instantiate each one
pub struct BuiltInEntities {}
impl BuiltInEntities {
    /// Associates each entity type with a key. Call once at initialization.
    pub fn register(mut factory: EntityFactory<dyn Entity>) -> EntityFactory<dyn Entity> {
        let include_internals = true;

        // Controllers
        factory.register_entity_with_str_key(SignalPassthroughController::ENTITY_KEY, |uid| {
            Box::new(SignalPassthroughController::new_with(uid))
        });
        factory.register_entity_with_str_key("signal-amplitude-passthrough", |uid| {
            Box::new(SignalPassthroughController::new_amplitude_passthrough_type(
                uid,
            ))
        });
        factory.register_entity_with_str_key("signal-amplitude-inverted-passthrough", |uid| {
            Box::new(SignalPassthroughController::new_amplitude_inverted_passthrough_type(uid))
        });
        if include_internals {
            factory.register_entity_with_str_key(Timer::ENTITY_KEY, |uid| {
                Box::new(Timer::new_with(uid, MusicalTime::DURATION_QUARTER))
            });
            factory.register_entity_with_str_key(Trigger::ENTITY_KEY, |uid| {
                Box::new(Trigger::new_with(
                    uid,
                    TimerCoreBuilder::default()
                        .duration(MusicalTime::DURATION_QUARTER)
                        .build()
                        .unwrap(),
                    ControlValue(1.0),
                ))
            });
        }

        // Effects
        factory.register_entity_with_str_key(Delay::ENTITY_KEY, |uid| {
            Box::new(Delay::new_with(
                uid,
                DelayCoreBuilder::default().build().unwrap(),
            ))
        });
        factory.register_entity_with_str_key(Gain::ENTITY_KEY, |uid| {
            Box::new(Gain::new_with(
                uid,
                GainCoreBuilder::default()
                    .ceiling(0.8.into())
                    .build()
                    .unwrap(),
            ))
        });
        factory.register_entity_with_str_key(Reverb::ENTITY_KEY, |uid| {
            Box::new(Reverb::new_with(
                uid,
                ReverbCoreBuilder::default()
                    .attenuation(0.8.into())
                    .seconds(1.0.into())
                    .build()
                    .unwrap(),
            ))
        });

        factory
    }
}
