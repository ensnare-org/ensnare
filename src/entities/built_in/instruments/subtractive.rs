// Copyright (c) 2024 Mike Tsao

use crate::{
    cores::{
        BiQuadFilterLowPass24dbCoreBuilder, LfoRouting, SubtractiveSynthCore,
        SubtractiveSynthCoreBuilder,
    },
    prelude::*,
};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument, InnerSerializable,
    IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

/// Entity wrapper for [SubtractiveSynthCore]
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
pub struct SubtractiveSynth {
    uid: Uid,
    inner: SubtractiveSynthCore,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    widget_action: Option<crate::egui::SubtractiveSynthWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl Default for SubtractiveSynth {
    fn default() -> Self {
        Self::new_with_factory_patch(Uid::default())
    }
}
impl SubtractiveSynth {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: SubtractiveSynthCore) -> Self {
        Self {
            uid,
            inner,
            #[cfg(feature = "egui")]
            widget_action: Default::default(),
            #[cfg(feature = "egui")]
            action: Default::default(),
        }
    }

    /// Creates new instance with default patch
    pub fn new_with_factory_patch(uid: Uid) -> Self {
        SubtractiveSynth::new_with(
            uid,
            SubtractiveSynthCoreBuilder::default()
                .oscillator_1(
                    OscillatorBuilder::default()
                        .waveform(Waveform::Sine)
                        .build()
                        .unwrap(),
                )
                .oscillator_2(
                    OscillatorBuilder::default()
                        .waveform(Waveform::Sawtooth)
                        .build()
                        .unwrap(),
                )
                .oscillator_2_sync(true)
                .oscillator_mix(0.8.into())
                .amp_envelope(EnvelopeBuilder::safe_default().build().unwrap())
                .dca(Dca::default())
                .lfo(
                    OscillatorBuilder::default()
                        .waveform(Waveform::Sine)
                        .frequency(0.2.into())
                        .build()
                        .unwrap(),
                )
                .lfo_routing(LfoRouting::FilterCutoff)
                .lfo_depth(Normal::from(0.5))
                .filter(
                    BiQuadFilterLowPass24dbCoreBuilder::default()
                        .cutoff(250.0.into())
                        .passband_ripple(1.0)
                        .build()
                        .unwrap(),
                )
                .filter_cutoff_start(Normal::from(0.1))
                .filter_cutoff_end(Normal::from(0.8))
                .filter_envelope(EnvelopeBuilder::safe_default().build().unwrap())
                .build()
                .unwrap(),
        )
    }

    /// Creates instance with built-in patch
    pub fn new_with_internal_patch(uid: Uid, patch_name: &str) -> anyhow::Result<Self> {
        let inner = SubtractiveSynthCore::load_internal_patch(patch_name)?;
        Ok(SubtractiveSynth::new_with(uid, inner))
    }
}

#[cfg(feature = "egui")]
impl crate::traits::Displays for SubtractiveSynth {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let response = ui.add(crate::egui::SubtractiveSynthWidget::widget(
            &mut self.inner,
            &mut self.widget_action,
        ));
        if let Some(action) = self.widget_action.take() {
            match action {
                crate::egui::SubtractiveSynthWidgetAction::Link(uid, index) => {
                    self.set_action(DisplaysAction::Link(uid, index));
                }
                crate::egui::SubtractiveSynthWidgetAction::LoadFromJson(name, json) => {
                    // TODO - this is just a hack. It's doing real work on
                    // the UI thread, and it doesn't handle failure well.
                    self.inner = SubtractiveSynthCore::load_patch_from_json(&json).unwrap();
                    self.inner.preset_name = Some(name);
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

// TODO: the feature=egui thing sucks. I haven't figured out an elegant way to
// group the egui parts, and having to qualify crate::traits::Displays was a
// very weird build problem that I discovered only by accident. I think the idea
// of gating egui stuff on a feature is fine, but I haven't implemented it well.
#[cfg(not(feature = "egui"))]
impl crate::traits::Displays for SubtractiveSynth {}
