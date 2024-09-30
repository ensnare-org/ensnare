// Copyright (c) 2024 Mike Tsao

use crate::{
    cores::{
        BiQuadFilterLowPass24dbCoreBuilder, DrumkitCore, FmSynthCore, FmSynthCoreBuilder,
        LfoRouting, SamplerCore, SubtractiveSynthCore, SubtractiveSynthCoreBuilder,
    },
    prelude::*,
    traits::DisplaysAction,
    util::{KitIndex, SampleSource},
};
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerHandlesMidi, InnerInstrument, InnerSerializable,
    IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "egui")]
use crate::egui::{
    DrumkitWidgetAction, FmSynthWidgetAction, SamplerWidgetAction, SubtractiveSynthWidgetAction,
};

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
    widget_action: Option<DrumkitWidgetAction>,

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
            widget_action: Default::default(),
            action: Default::default(),
        }
    }

    /// Reads kit of samples from disk
    pub fn load(&mut self) -> anyhow::Result<()> {
        self.inner.load()
    }
}

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
    widget_action: Option<FmSynthWidgetAction>,

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
            widget_action: Default::default(),
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
            widget_action: Default::default(),
            action: Default::default(),
        }
    }

    /// Reads sample from disk
    pub fn load(&mut self) -> anyhow::Result<()> {
        self.inner.load()
    }
}

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
    widget_action: Option<SubtractiveSynthWidgetAction>,

    #[cfg(feature = "egui")]
    #[serde(skip)]
    action: Option<DisplaysAction>,
}
impl SubtractiveSynth {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid, inner: SubtractiveSynthCore) -> Self {
        Self {
            uid,
            inner,
            widget_action: Default::default(),
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
mod egui {
    use super::*;
    use crate::{
        egui::{DrumkitWidget, FmSynthWidget, SamplerWidget, SubtractiveSynthWidget},
        traits::DisplaysAction,
    };

    impl Displays for Drumkit {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let response = ui.add(DrumkitWidget::widget(
                &mut self.inner,
                &mut self.widget_action,
            ));
            if let Some(action) = self.widget_action.take() {
                match action {
                    DrumkitWidgetAction::Link(payload, index) => {
                        self.set_action(DisplaysAction::Link(payload, index));
                    }
                    DrumkitWidgetAction::Load(kit_index) => self.inner.set_kit_index(kit_index),
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

    impl Displays for FmSynth {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let response = ui.add(FmSynthWidget::widget(
                &mut self.inner,
                &mut self.widget_action,
            ));
            if let Some(action) = self.widget_action.take() {
                match action {
                    FmSynthWidgetAction::Link(source, index) => {
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

    impl Displays for Sampler {
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

    impl Displays for SubtractiveSynth {
        fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
            let response = ui.add(SubtractiveSynthWidget::widget(
                &mut self.inner,
                &mut self.widget_action,
            ));
            if let Some(action) = self.widget_action.take() {
                match action {
                    SubtractiveSynthWidgetAction::Link(uid, index) => {
                        self.set_action(DisplaysAction::Link(uid, index));
                    }
                    SubtractiveSynthWidgetAction::LoadFromJson(name, json) => {
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
}
