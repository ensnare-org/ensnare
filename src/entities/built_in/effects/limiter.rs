// Copyright (c) 2024 Mike Tsao

use crate::{cores::LimiterCore, prelude::*};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// Entity wrapper for [LimiterCore]
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
pub struct Limiter {
    uid: Uid,
    inner: LimiterCore,
}
impl Limiter {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: LimiterCore) -> Self {
        Self { uid, inner }
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for Limiter {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut min = self.inner.minimum().to_percentage();
        let mut max = self.inner.maximum().to_percentage();
        let min_response = ui.add(
            eframe::egui::Slider::new(&mut min, 0.0..=max)
                .suffix(" %")
                .text("min")
                .fixed_decimals(2),
        );
        if min_response.changed() {
            self.inner.set_minimum(min.into());
        };
        let max_response = ui.add(
            eframe::egui::Slider::new(&mut max, min..=1.0)
                .suffix(" %")
                .text("max")
                .fixed_decimals(2),
        );
        if max_response.changed() {
            self.inner.set_maximum(Normal::from_percentage(max));
        };
        min_response | max_response
    }
}
#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for Limiter {}
