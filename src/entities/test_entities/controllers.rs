// Copyright (c) 2024 Mike Tsao

use crate::{
    cores::{SimpleControllerAlwaysSendsMidiMessageCore, TimerCore},
    prelude::*,
};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerControls, InnerHandlesMidi, InnerSerializable,
    IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// The smallest possible [IsEntity].
#[derive(Debug, Default, IsEntity, Metadata, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
pub struct TestController {
    uid: Uid,
}
impl TestController {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid) -> Self {
        Self { uid }
    }
}

/// Wraps [SimpleControllerAlwaysSendsMidiMessageCore]
#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControllable,
    InnerControls,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Displays, GeneratesStereoSample, SkipInner, TransformsAudio)]
pub struct TestControllerAlwaysSendsMidiMessage {
    uid: Uid,
    #[serde(skip)]
    inner: SimpleControllerAlwaysSendsMidiMessageCore,
}
impl TestControllerAlwaysSendsMidiMessage {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: SimpleControllerAlwaysSendsMidiMessageCore::default(),
        }
    }
}

/// Wraps [Timer]
#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControllable,
    InnerControls,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Displays, GeneratesStereoSample, SkipInner, TransformsAudio)]
pub struct TestControllerTimed {
    uid: Uid,
    #[serde(skip)]
    inner: TimerCore,
}
impl TestControllerTimed {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, duration: MusicalTime) -> Self {
        Self {
            uid,
            inner: TimerCore::new_with(duration),
        }
    }
}
