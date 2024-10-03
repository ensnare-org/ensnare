// Copyright (c) 2024 Mike Tsao

//! Simple instruments, controllers, and effects. Simple entities are minimal
//! but musically useful. Their main purpose is to facilitate development.

pub use controllers::SimpleController;
pub use effects::SimpleEffect;
pub use factory::SimpleEntities;
pub use instruments::SimpleInstrument;

mod controllers;
mod effects;
mod factory;
mod instruments;
