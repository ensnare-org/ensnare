// Copyright (c) 2024 Mike Tsao

use crate::{
    cores::{FmSynthCore, FmSynthCoreBuilder},
    prelude::*,
};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument, InnerSerializable,
    IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// Entity wrapper for [FmSynthCore]
#[derive(
    Debug,
    InnerConfigurable,
    InnerControllable,
    InnerHandlesMidi,
    InnerInstrument,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
#[entity(Controls, TransformsAudio)]
pub struct FmSynth {
    uid: Uid,
    inner: FmSynthCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<crate::egui::FmSynthWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl FmSynth {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: FmSynthCore) -> Self {
        Self {
            uid,
            inner,
            #[cfg(feature = "egui")]
            widget_action: Default::default(),
            #[cfg(feature = "egui")]
            action: Default::default(),
        }
    }

    /// TODO: reduce to pub(crate)
    // A crisp, classic FM sound that brings me back to 1985.
    pub fn new_with_factory_patch(uid: Uid) -> Self {
        Self::new_with(
            uid,
            FmSynthCoreBuilder::default()
                .carrier(
                    OscillatorBuilder::default()
                        .waveform(Waveform::Sine)
                        .build()
                        .unwrap(),
                )
                .carrier_envelope(
                    EnvelopeBuilder::default()
                        .attack(0.0001.into())
                        .decay(0.0005.into())
                        .sustain(0.60.into())
                        .release(0.25.into())
                        .build()
                        .unwrap(),
                )
                .modulator(
                    OscillatorBuilder::default()
                        .waveform(Waveform::Sine)
                        .build()
                        .unwrap(),
                )
                .modulator_envelope(
                    EnvelopeBuilder::default()
                        .attack(0.0001.into())
                        .decay(0.0005.into())
                        .sustain(0.30.into())
                        .release(0.25.into())
                        .build()
                        .unwrap(),
                )
                .depth(0.35.into())
                .ratio(4.5.into())
                .beta(40.0.into())
                .dca(Dca::default())
                .build()
                .unwrap(),
        )
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for FmSynth {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = ui.add(crate::egui::FmSynthWidget::widget(
            &mut self.inner,
            &mut self.widget_action,
        ));
        if let Some(action) = self.widget_action.take() {
            match action {
                crate::egui::FmSynthWidgetAction::Link(source, index) => {
                    self.set_action(DisplaysAction::Link(source, index));
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
impl crate::traits::Displays for FmSynth {}
