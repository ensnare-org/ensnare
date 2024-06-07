// Copyright (c) 2024 Mike Tsao

use crate::{
    cores::{DelayCore, GainCore, ReverbCore},
    prelude::*,
};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

mod delay;
mod gain;
mod reverb;

/// Wraps [DelayCore] and makes it an [Entity].
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

pub struct Delay {
    uid: Uid,
    inner: DelayCore,
}
impl Delay {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: DelayCore) -> Self {
        Self { uid, inner }
    }
}

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

/// Wraps [GainCore] and makes it an [Entity].
pub struct Gain {
    uid: Uid,
    inner: GainCore,
}
impl Gain {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: GainCore) -> Self {
        Self { uid, inner }
    }
}

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
