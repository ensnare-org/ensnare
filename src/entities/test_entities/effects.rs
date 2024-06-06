// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use ensnare_proc_macros::{IsEntity, Metadata};
use serde::{Deserialize, Serialize};

/// The smallest possible [IsEntity] that acts like an effect.
#[derive(Debug, Default, IsEntity, Metadata, Serialize, Deserialize)]
#[entity(
    Configurable,
    Controllable,
    Controls,
    Displays,
    GeneratesStereoSample,
    HandlesMidi,
    Serializable,
    SkipInner,
    TransformsAudio
)]
#[serde(rename_all = "kebab-case")]
pub struct TestEffect {
    uid: Uid,
}
impl TestEffect {
    pub fn new_with(uid: Uid) -> Self {
        Self { uid }
    }
}
