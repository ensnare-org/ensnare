// Copyright (c) 2024 Mike Tsao

use crate::{
    cores::{TimerCore, TimerCoreBuilder},
    prelude::*,
};
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerControls, InnerHandlesMidi, InnerSerializable, IsEntity,
    Metadata,
};
use serde::{Deserialize, Serialize};

/// Wraps [TimerCore] and makes it an [Entity].
#[derive(
    Control,
    Debug,
    InnerConfigurable,
    InnerControls,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "kebab-case")]
#[entity(GeneratesStereoSample, TransformsAudio)]
pub struct Timer {
    uid: Uid,
    inner: TimerCore,
}
impl Timer {
    /// Creates a new [Timer].
    pub fn new_with(uid: Uid, duration: MusicalTime) -> Self {
        Self {
            uid,
            inner: TimerCoreBuilder::default()
                .duration(duration)
                .build()
                .unwrap(),
        }
    }
}

#[cfg(feature = "egui")]
mod egui {
    use super::*;
    impl Displays for Timer {}
}
#[cfg(not(feature = "egui"))]
mod egui {
    use super::*;
    use crate::traits::Displays;
    impl Displays for Timer {}
}
