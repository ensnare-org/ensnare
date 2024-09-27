// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// An effect that negates the input.
#[derive(Debug, Default, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TestEffectNegatesInputCore {}
impl TransformsAudio for TestEffectNegatesInputCore {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        -input_sample
    }
}
