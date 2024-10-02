// Copyright (c) 2024 Mike Tsao

use crate::{cores::LfoControllerCore, prelude::*};
use ensnare_proc_macros::{
    Control, InnerConfigurable, InnerControls, InnerHandlesMidi, InnerSerializable, IsEntity,
    Metadata,
};
use serde::{Deserialize, Serialize};

/// Entity wrapper for [LfoControllerCore]
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
pub struct LfoController {
    uid: Uid,
    inner: LfoControllerCore,
}
impl LfoController {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: LfoControllerCore) -> Self {
        Self { uid, inner }
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for LfoController {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = ui.add(crate::egui::LfoControllerWidget::widget(
            &mut self.inner.oscillator.waveform,
            &mut self.inner.oscillator.frequency,
        ));
        if response.changed() {
            self.inner.notify_change_oscillator();
        }
        response
    }
}
#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for LfoController {}
