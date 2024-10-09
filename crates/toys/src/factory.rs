// Copyright (c) 2024 Mike Tsao

use crate::{controllers::*, effects::*, instruments::*};
use ensnare::prelude::*;

/// Registers toy entities for the given [EntityFactory]. Toy entities are very
/// simple but working instruments. They're helpful when you think you're going
/// nuts because nothing is working, so you want something that doesn't have
/// lots of settings.
pub struct ToyEntities {}
impl ToyEntities {
    /// Registers all the entities in this collection.
    pub fn register(mut factory: EntityFactory<dyn Entity>) -> EntityFactory<dyn Entity> {
        factory.register_entity(EntityKey::from(ToySynth::ENTITY_KEY), |uid| {
            Box::new(ToySynth::new_with(
                uid,
                OscillatorBuilder::default()
                    .waveform(Waveform::Triangle)
                    .build()
                    .unwrap(),
                EnvelopeBuilder::safe_default().build().unwrap(),
                Dca::default(),
            ))
        });
        factory.register_entity(EntityKey::from(ToyInstrument::ENTITY_KEY), |uid| {
            Box::new(ToyInstrument::new_with(uid))
        });
        factory.register_entity(EntityKey::from(ToyController::ENTITY_KEY), |uid| {
            Box::new(ToyController::new_with(uid))
        });
        factory.register_entity(EntityKey::from(ToyEffect::ENTITY_KEY), |uid| {
            Box::new(ToyEffect::new_with(uid, 0.8.into()))
        });

        factory
    }
}
