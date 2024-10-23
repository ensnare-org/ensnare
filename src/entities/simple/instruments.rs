// Copyright (c) 2024 Mike Tsao

use crate::{
    cores::{SimpleOscillatorCore, SimpleOscillatorCoreBuilder},
    prelude::*,
};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerInstrument, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// An instrument that produces a simple audio signal but doesn't respond to
/// MIDI events.
#[derive(
    Debug,
    Default,
    Deserialize,
    InnerConfigurable,
    InnerControllable,
    InnerInstrument,
    IsEntity,
    Metadata,
    Serialize,
)]
#[entity(Controls, Displays, TransformsAudio)]
pub struct SimpleInstrumentDrone {
    uid: Uid,
    inner: SimpleOscillatorCore,
}
impl SimpleInstrumentDrone {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: SimpleOscillatorCoreBuilder::default().build().unwrap(),
        }
    }
}
impl HandlesMidi for SimpleInstrumentDrone {}
impl Serializable for SimpleInstrumentDrone {}
