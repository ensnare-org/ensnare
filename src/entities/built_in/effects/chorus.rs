// Copyright (c) 2024 Mike Tsao

use crate::{cores::ChorusCore, prelude::*};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// Entity wrapper for [ChorusCore]
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

pub struct Chorus {
    uid: Uid,
    inner: ChorusCore,
}
impl Chorus {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: ChorusCore) -> Self {
        Self { uid, inner }
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for Chorus {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.label("Coming soon!")
    }
}
#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for Chorus {}
