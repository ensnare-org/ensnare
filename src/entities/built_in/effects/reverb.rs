// Copyright (c) 2024 Mike Tsao

use crate::{cores::ReverbCore, prelude::*};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// Wraps [ReverbCore] and makes it an [Entity].
#[derive(
    Debug,
    Default,
    InnerControllable,
    InnerConfigurable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Controls, GeneratesStereoSample, HandlesMidi, SkipInner)]

/// Entity wrapper for [ReverbCore]
pub struct Reverb {
    uid: Uid,
    inner: ReverbCore,
}
impl Reverb {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: ReverbCore) -> Self {
        Self { uid, inner }
    }
}
impl crate::traits::Displays for Reverb {}
