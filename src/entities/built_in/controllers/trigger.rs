// Copyright (c) 2024 Mike Tsao

use crate::{
    cores::{TimerCore, TriggerCore, TriggerCoreBuilder},
    prelude::*,
};
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerControls, InnerHandlesMidi, InnerSerializable, IsEntity,
    Metadata,
};
use serde::{Deserialize, Serialize};

/// Wraps [TriggerCore] and makes it an [Entity].
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
pub struct Trigger {
    uid: Uid,
    inner: TriggerCore,
}
impl Trigger {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, timer: TimerCore, value: ControlValue) -> Self {
        Self {
            uid,
            inner: TriggerCoreBuilder::default()
                .timer(timer)
                .value(value)
                .build()
                .unwrap(),
        }
    }
}

#[cfg(feature = "egui")]
mod egui {
    use super::*;
    impl Displays for Trigger {}
}
#[cfg(not(feature = "egui"))]
mod egui {
    use super::*;
    use crate::traits::Displays;
    impl Displays for Trigger {}
}
