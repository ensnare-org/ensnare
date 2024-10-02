// Copyright (c) 2024 Mike Tsao

//! Toy Entities that wrap the corresponding toy devices.

/// Recommended imports for easy onboarding.
pub mod prelude {
    pub use super::controllers::*;
    pub use super::effects::*;
    pub use super::instruments::*;
    pub use super::ToyEntities;
}

pub use crate::controllers::*;
pub use crate::effects::*;
pub use crate::instruments::*;
pub use factory::ToyEntities;

mod controllers;
mod cores;
mod effects;
mod factory;
mod instruments;
