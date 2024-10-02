// Copyright (c) 2024 Mike Tsao

use crate::cores::ToyEffectCore;
use ensnare::{egui::DragNormalWidget, prelude::*};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerEffect, InnerSerializable, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControllable,
    InnerEffect,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(HandlesMidi, GeneratesStereoSample, Controls)]
pub struct ToyEffect {
    uid: Uid,
    inner: ToyEffectCore,
}
impl Displays for ToyEffect {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(DragNormalWidget::widget(
            &mut self.inner.magnitude,
            "Magnitude: ",
        ))
    }
}
impl ToyEffect {
    pub fn new_with(uid: Uid, magnitude: Normal) -> Self {
        Self {
            uid,
            inner: ToyEffectCore::new_with(magnitude),
        }
    }
}
