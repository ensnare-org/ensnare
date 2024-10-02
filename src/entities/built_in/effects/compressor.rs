// Copyright (c) 2024 Mike Tsao

use crate::{cores::CompressorCore, prelude::*};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// Entity wrapper for [CompressorCore]
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

pub struct Compressor {
    uid: Uid,
    inner: CompressorCore,
}
impl Compressor {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: CompressorCore) -> Self {
        Self { uid, inner }
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for Compressor {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut threshold = self.inner.threshold().0;
        let mut ratio = self.inner.ratio().0;
        let mut attack = self.inner.attack().0;
        let mut release = self.inner.release().0;
        let threshold_response = ui.add(
            eframe::egui::Slider::new(&mut threshold, Normal::range())
                .fixed_decimals(2)
                .text("Threshold"),
        );
        if threshold_response.changed() {
            self.inner.set_threshold(threshold.into());
        };
        let ratio_response = ui.add(
            eframe::egui::Slider::new(&mut ratio, 0.05..=2.0)
                .fixed_decimals(2)
                .text("Ratio"),
        );
        if ratio_response.changed() {
            self.inner.set_ratio(ratio.into());
        };
        let attack_response = ui.add(
            eframe::egui::Slider::new(&mut attack, Normal::range())
                .fixed_decimals(2)
                .text("Attack"),
        );
        if attack_response.changed() {
            self.inner.set_attack(attack.into());
        };
        let release_response = ui.add(
            eframe::egui::Slider::new(&mut release, Normal::range())
                .fixed_decimals(2)
                .text("Release"),
        );
        if release_response.changed() {
            self.inner.set_release(release.into());
        };
        threshold_response | ratio_response | attack_response | release_response
    }
}
#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for Compressor {}
