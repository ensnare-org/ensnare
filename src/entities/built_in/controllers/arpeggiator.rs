// Copyright (c) 2024 Mike Tsao

use crate::{cores::ArpeggiatorCore, prelude::*};
use ensnare_proc_macros::{Control, InnerConfigurable, InnerHandlesMidi, IsEntity, Metadata};
use serde::{Deserialize, Serialize};

/// Entity wrapper for [ArpeggiatorCore]
#[derive(
    Debug,
    Default,
    Control,
    InnerConfigurable,
    InnerHandlesMidi,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "kebab-case")]
#[entity(Controls, GeneratesStereoSample, Serializable, TransformsAudio)]
pub struct Arpeggiator {
    uid: Uid,
    inner: ArpeggiatorCore,
}
impl Arpeggiator {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: ArpeggiatorCore) -> Self {
        Self { uid, inner }
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for Arpeggiator {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.add(crate::egui::ArpeggiatorWidget::widget(&mut self.inner))
    }
}
#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for Arpeggiator {}
