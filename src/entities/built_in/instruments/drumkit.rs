// Copyright (c) 2024 Mike Tsao

use crate::{cores::DrumkitCore, prelude::*, traits::DisplaysAction, util::KitIndex};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument, IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// Entity wrapper for [DrumkitCore]
#[derive(
    Debug,
    InnerControllable,
    InnerConfigurable,
    InnerHandlesMidi,
    InnerInstrument,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "kebab-case")]
#[entity(Controls, Serializable, TransformsAudio)]

pub struct Drumkit {
    uid: Uid,
    inner: DrumkitCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<crate::egui::DrumkitWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl Drumkit {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, kit_index: KitIndex) -> Self {
        Self {
            uid,
            inner: DrumkitCore::new_with_kit_index(kit_index),
            #[cfg(feature = "egui")]
            widget_action: Default::default(),
            #[cfg(feature = "egui")]
            action: Default::default(),
        }
    }

    /// Reads kit of samples from disk
    pub fn load(&mut self) -> anyhow::Result<()> {
        self.inner.load()
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for Drumkit {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = ui.add(crate::egui::DrumkitWidget::widget(
            &mut self.inner,
            &mut self.widget_action,
        ));
        if let Some(action) = self.widget_action.take() {
            match action {
                crate::egui::DrumkitWidgetAction::Link(payload, index) => {
                    self.set_action(DisplaysAction::Link(payload, index));
                }
                crate::egui::DrumkitWidgetAction::Load(kit_index) => {
                    self.inner.set_kit_index(kit_index)
                }
            }
        }
        response
    }

    fn set_action(&mut self, action: DisplaysAction) {
        self.action = Some(action);
    }

    fn take_action(&mut self) -> Option<DisplaysAction> {
        self.action.take()
    }
}

#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for Drumkit {}
