// Copyright (c) 2024 Mike Tsao

//! egui widgets and components

/// A collection of imports that are useful for users of this crate who are also using egui.
pub mod prelude {
    pub use super::util::fill_remaining_ui_space;
}

pub use audio::*;
pub use automation::*;
pub use controllers::*;
pub use effects::*;
pub use generators::*;
pub use glue::*;
pub use instruments::*;
pub use modulators::*;
pub use util::*;

mod audio;
mod automation;
mod controllers;
mod effects;
mod generators;
mod glue;
mod instruments;
mod modulators;
mod util;
