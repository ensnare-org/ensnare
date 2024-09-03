// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// An effect that multiplies the input by 0.5, which is basically a gain set to 50%.
#[derive(Debug, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SimpleEffectHalfCore {}
impl TransformsAudio for SimpleEffectHalfCore {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        input_sample * 0.5
    }
}
impl Serializable for SimpleEffectHalfCore {}
impl Configurable for SimpleEffectHalfCore {}
