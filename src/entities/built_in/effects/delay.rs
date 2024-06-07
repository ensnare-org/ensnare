// Copyright (c) 2024 Mike Tsao

#[cfg(feature = "egui")]
mod egui {
    use crate::{entities::Delay, prelude::Displays};

    impl Displays for Delay {}
}
