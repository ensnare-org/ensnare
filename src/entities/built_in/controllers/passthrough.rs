// Copyright (c) 2024 Mike Tsao

use crate::{
    cores::{SignalPassthroughControllerCore, SignalPassthroughControllerCoreBuilder},
    prelude::*,
};
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerControls, InnerHandlesMidi, InnerSerializable, IsEntity,
    Metadata,
};
use serde::{Deserialize, Serialize};

/// Wraps [SignalPassthroughControllerCore] and makes it an [Entity].
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
pub struct SignalPassthroughController {
    uid: Uid,
    inner: SignalPassthroughControllerCore,
}
impl SignalPassthroughController {
    /// Creates a new [SignalPassthroughController] configured as Compressed.
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: SignalPassthroughControllerCoreBuilder::default()
                .build()
                .unwrap(),
        }
    }

    /// Creates a new [SignalPassthroughController] configured as Amplitude.
    pub fn new_amplitude_passthrough_type(uid: Uid) -> Self {
        Self {
            uid,
            inner: SignalPassthroughControllerCoreBuilder::amplitude()
                .build()
                .unwrap(),
        }
    }

    /// Creates a new [SignalPassthroughController] configured as AmplitudeInverted.
    pub fn new_amplitude_inverted_passthrough_type(uid: Uid) -> Self {
        Self {
            uid,
            inner: SignalPassthroughControllerCoreBuilder::amplitude_inverted()
                .build()
                .unwrap(),
        }
    }
}
#[cfg(feature = "egui")]
mod egui {
    use super::*;
    impl Displays for SignalPassthroughController {}
}
#[cfg(not(feature = "egui"))]
mod egui {
    use super::*;
    use crate::traits::Displays;
    impl Displays for SignalPassthroughController {}
}
