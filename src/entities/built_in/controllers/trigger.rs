// Copyright (c) 2024 Mike Tsao

use crate::{
    cores::{TimerCore, TriggerCore},
    prelude::*,
};
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerControls, InnerHandlesMidi, InnerSerializable, IsEntity,
    Metadata,
};
use serde::{Deserialize, Serialize};

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
    pub fn new_with(uid: Uid, timer: TimerCore, value: ControlValue) -> Self {
        Self {
            uid,
            inner: TriggerCore::new_with(timer, value),
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
