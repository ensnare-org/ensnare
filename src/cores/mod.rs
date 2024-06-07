// Copyright (c) 2024 Mike Tsao

//! Basic musical devices without the overhead that the rest of the system needs
//! to use them. A core plus that overhead is an
//! [Entity][crate::traits::Entity]. Cores exist separately from entities so
//! that it's easier to focus on business logic when developing a new device.

pub use controllers::*;
pub use effects::*;
pub use instruments::*;

mod controllers;
mod effects;
mod instruments;
