// Copyright (c) 2024 Mike Tsao

use super::{
    controllers::TestController,
    effects::TestEffect,
    instruments::{TestInstrument, TestInstrumentCountsMidiMessages},
};
use crate::prelude::*;

/// A collection of test entities.
pub struct TestEntities {}
impl CollectsEntities for TestEntities {
    #[must_use]
    fn register(mut factory: EntityFactory<dyn Entity>) -> EntityFactory<dyn Entity> {
        factory.register_entity_with_str_key(TestInstrument::ENTITY_KEY, |_uid| {
            Box::new(TestInstrument::default())
        });
        factory
            .register_entity_with_str_key(TestInstrumentCountsMidiMessages::ENTITY_KEY, |_uid| {
                Box::new(TestInstrumentCountsMidiMessages::default())
            });
        factory.register_entity_with_str_key(TestController::ENTITY_KEY, |_uid| {
            Box::new(TestController::default())
        });
        factory.register_entity_with_str_key(TestEffect::ENTITY_KEY, |_uid| {
            Box::new(TestEffect::default())
        });

        factory
    }
}
