// Copyright (c) 2024 Mike Tsao

use crate::{
    cores::{SimpleNoisyAudioSourceCore, SimpleNoisyAudioSourceCoreBuilder},
    prelude::*,
};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerInstrument, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// An instrument that produces a constant audio signal.
#[derive(
    Debug,
    Default,
    Deserialize,
    InnerConfigurable,
    InnerControllable,
    InnerInstrument,
    IsEntity,
    Metadata,
    Serialize,
)]
#[entity(Controls, TransformsAudio)]
pub struct SimpleInstrument {
    uid: Uid,
    inner: SimpleNoisyAudioSourceCore,
}
impl Displays for SimpleInstrument {}
impl SimpleInstrument {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: SimpleNoisyAudioSourceCoreBuilder::default()
                .build()
                .unwrap(),
        }
    }
}
impl HandlesMidi for SimpleInstrument {}
impl Serializable for SimpleInstrument {}
