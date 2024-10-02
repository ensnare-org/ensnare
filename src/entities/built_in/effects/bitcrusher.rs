// Copyright (c) 2024 Mike Tsao

use crate::{cores::BitcrusherCore, prelude::*};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// Entity wrapper for [BitcrusherCore]
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
pub struct Bitcrusher {
    uid: Uid,
    inner: BitcrusherCore,
}
impl Bitcrusher {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: BitcrusherCore) -> Self {
        Self { uid, inner }
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for Bitcrusher {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut bits = self.inner.bits();
        let response = ui.add(
            eframe::egui::Slider::new(&mut bits, BitcrusherCore::bits_range()).suffix(" bits"),
        );
        if response.changed() {
            self.inner.set_bits(bits);
        };
        response
    }
}
#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for Bitcrusher {}
