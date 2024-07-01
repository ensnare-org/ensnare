// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use ensnare_proc_macros::{IsEntity, Metadata};
use serde::{Deserialize, Serialize};

/// The smallest possible [IsEntity] that acts like an instrument.
#[derive(Debug, Default, IsEntity, Metadata, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[entity(
    Controllable,
    Controls,
    Displays,
    HandlesMidi,
    Serializable,
    SkipInner,
    TransformsAudio
)]

/// A simple instrument that does nothing.
#[allow(missing_docs)]
pub struct SomethingInstrument {
    pub uid: Uid,
    pub sample_rate: SampleRate,
}
impl SomethingInstrument {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            ..Default::default()
        }
    }
}
impl Configurable for SomethingInstrument {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
    }
}
impl Generates<StereoSample> for SomethingInstrument {}
