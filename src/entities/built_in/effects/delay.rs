// Copyright (c) 2024 Mike Tsao

use crate::{cores::DelayCore, prelude::*};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

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

/// Entity wrapper for [DelayCore]
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

#[cfg(feature = "egui")]
mod egui {
    use super::Delay;
    use crate::prelude::Displays;

    impl Displays for Delay {}
}
