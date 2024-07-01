// Copyright (c) 2024 Mike Tsao

use super::{controllers::*, effects::*, instruments::*};
use crate::prelude::*;

/// Registers all [EntityFactory]'s entities.
#[must_use]
pub fn register_simple_entities(
    mut factory: EntityFactory<dyn Entity>,
) -> EntityFactory<dyn Entity> {
    factory.register_entity_with_str_key(SimpleController::ENTITY_KEY, |_uid| {
        Box::new(SimpleController::default())
    });
    factory.register_entity_with_str_key(SimpleEffect::ENTITY_KEY, |_uid| {
        Box::new(SimpleEffect::default())
    });
    factory.register_entity_with_str_key(SimpleInstrument::ENTITY_KEY, |_uid| {
        Box::new(SimpleInstrument::default())
    });

    factory
}
