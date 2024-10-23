// Copyright (c) 2024 Mike Tsao

use super::{controllers::*, effects::*, instruments::*};
use crate::prelude::*;

/// A collection of simple entities that are not musically useful, but are
/// useful for known-good behavior during development.
pub struct SimpleEntities {}
impl CollectsEntities for SimpleEntities {
    #[must_use]
    fn register(mut factory: EntityFactory<dyn Entity>) -> EntityFactory<dyn Entity> {
        factory
            .register_entity_with_str_key(SimpleControllerOneNoteOneMeasure::ENTITY_KEY, |_uid| {
                Box::new(SimpleControllerOneNoteOneMeasure::default())
            });
        factory.register_entity_with_str_key(SimpleEffect::ENTITY_KEY, |_uid| {
            Box::new(SimpleEffect::default())
        });
        factory.register_entity_with_str_key(SimpleInstrumentDrone::ENTITY_KEY, |_uid| {
            Box::new(SimpleInstrumentDrone::default())
        });

        factory
    }
}
