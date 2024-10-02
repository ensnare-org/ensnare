// Copyright (c) 2024 Mike Tsao

use crate::{cores::SimpleEffectHalfCore, prelude::*};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// An effect that negates the input.
#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControllable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Controls, Displays, GeneratesStereoSample, HandlesMidi)]
pub struct SimpleEffect {
    uid: Uid,
    inner: SimpleEffectHalfCore,
}
