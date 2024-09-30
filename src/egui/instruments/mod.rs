// Copyright (c) 2024 Mike Tsao

pub use {
    drumkit::{DrumkitWidget, DrumkitWidgetAction},
    fm::{FmSynthWidget, FmSynthWidgetAction},
    sampler::{SamplerWidget, SamplerWidgetAction},
    subtractive::{SubtractiveSynthWidget, SubtractiveSynthWidgetAction},
};

mod drumkit;
mod fm;
mod sampler;
mod subtractive;
