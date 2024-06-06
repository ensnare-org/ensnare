// Copyright (c) 2024 Mike Tsao

use super::Trigger;
use crate::{cores::TimerCore, prelude::*};

/// A collection of all entities that are suitable for normal use. Allows the
/// creation of an [EntityFactory] that lets apps refer to entities by
/// [EntityKey] rather than having to import and instantiate each one
pub struct BuiltInEntities {}
impl BuiltInEntities {
    /// Associates each entity type with a key. Call once at initialization.
    pub fn register(mut factory: EntityFactory<dyn Entity>) -> EntityFactory<dyn Entity> {
        let include_internals = false;

        // Controllers
        if include_internals {
            factory.register_entity_with_str_key(Timer::ENTITY_KEY, |uid| {
                Box::new(Timer::new_with(uid, MusicalTime::DURATION_QUARTER))
            });
            factory.register_entity_with_str_key(Trigger::ENTITY_KEY, |uid| {
                Box::new(Trigger::new_with(
                    uid,
                    TimerCore::new_with(MusicalTime::DURATION_QUARTER),
                    ControlValue(1.0),
                ))
            });
        }
        factory
    }
}
