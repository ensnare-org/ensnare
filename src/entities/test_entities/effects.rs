// Copyright (c) 2024 Mike Tsao

use crate::{cores::SimpleEffectHalfCore, prelude::*};
use ensnare_proc_macros::{InnerControllable, InnerTransformsAudio, IsEntity, Metadata};
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
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid) -> Self {
        Self { uid }
    }
}

/// Flips the sign of every audio sample it sees.
#[derive(
    Debug,
    Default,
    IsEntity,
    InnerControllable,
    InnerTransformsAudio,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(
    Configurable,
    Controls,
    Displays,
    GeneratesStereoSample,
    HandlesMidi,
    Serializable,
    SkipInner
)]
#[serde(rename_all = "kebab-case")]
pub struct TestEffectNegatesInput {
    uid: Uid,
    inner: SimpleEffectHalfCore,
}
impl TestEffectNegatesInput {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: Default::default(),
        }
    }
}
