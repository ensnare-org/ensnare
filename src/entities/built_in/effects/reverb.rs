// Copyright (c) 2024 Mike Tsao

#[cfg(feature = "egui")]
mod egui {
    use crate::{entities::Reverb, prelude::Displays};

    impl Displays for Reverb {}
}
