// Copyright (c) 2024 Mike Tsao

use crate::{cores::SamplerCore, prelude::*, traits::DisplaysAction, util::SampleSource};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument, InnerSerializable,
    IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "egui")]
use crate::egui::SamplerWidgetAction;

/// Entity wrapper for [SamplerCore]
#[derive(
    Debug,
    Deserialize,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
)]
#[entity(Controls, TransformsAudio)]
pub struct Sampler {
    uid: Uid,
    inner: SamplerCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<SamplerWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl Sampler {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, source: SampleSource, root: Option<FrequencyHz>) -> Self {
        Self {
            uid,
            inner: SamplerCore::new_with(source, root),
            #[cfg(feature = "egui")]
            widget_action: Default::default(),
            #[cfg(feature = "egui")]
            action: Default::default(),
        }
    }

    /// Reads sample from disk
    pub fn load(&mut self) -> anyhow::Result<()> {
        self.inner.load()
    }
}

#[cfg(feature = "egui")]
mod egui {
    use super::*;
    use crate::{egui::SamplerWidget, traits::DisplaysAction};

    impl crate::traits::Displays for Sampler {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let response = ui.add(SamplerWidget::widget(
                &mut self.inner,
                &mut self.widget_action,
            ));
            if let Some(action) = self.widget_action.take() {
                match action {
                    SamplerWidgetAction::Link(source, index) => {
                        self.set_action(DisplaysAction::Link(source, index));
                    }
                    SamplerWidgetAction::Load(index) => {
                        self.inner.set_source(SampleSource::SampleLibrary(index));
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
}

#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for Sampler {}
