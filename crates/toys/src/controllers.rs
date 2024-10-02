// Copyright (c) 2024 Mike Tsao

use crate::cores::{ToyControllerCore, ToySequencerCore};
use eframe::egui::Slider;
use ensnare::prelude::*;
use ensnare_proc_macros::{
    InnerConfigurable, InnerControllable, InnerControls, InnerHandlesMidi, InnerSerializable,
    IsEntity, Metadata,
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControls,
    InnerControllable,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
pub struct ToyController {
    uid: Uid,
    #[serde(skip)]
    inner: ToyControllerCore,
}
impl Generates<StereoSample> for ToyController {}
impl TransformsAudio for ToyController {}
impl Displays for ToyController {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut channel = self.inner.midi_channel_out.0;
        let slider_response = ui.add(Slider::new(&mut channel, 0..=15).text("MIDI out"));
        if slider_response.changed() {
            self.inner.midi_channel_out = MidiChannel(channel);
        }
        ui.end_row();
        slider_response | ui.checkbox(&mut self.inner.is_enabled, "Enabled")
    }
}
impl ToyController {
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: ToyControllerCore::new_with(MidiChannel::default()),
        }
    }
}

#[derive(
    Debug,
    Default,
    InnerConfigurable,
    InnerControls,
    InnerControllable,
    InnerHandlesMidi,
    InnerSerializable,
    IsEntity,
    Metadata,
    Serialize,
    Deserialize,
)]
pub struct ToySequencer {
    uid: Uid,
    #[serde(skip)]
    inner: ToySequencerCore,
}
impl Generates<StereoSample> for ToySequencer {}
impl TransformsAudio for ToySequencer {}
impl Displays for ToySequencer {}
impl ToySequencer {
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: ToySequencerCore::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // TODO: I wasn't able to figure out a clean way for the main crate to
    // provide test-only functionality for other crates to use. This is
    // desirable because authors of external crates will want to know whether
    // their entities conform to our expectations of crate behavior.

    // #[test]
    // fn toy_passes_sequences_trait_validation() {
    //     let mut s = ToySequencer::default();

    //     validate_sequences_midi_trait(&mut s);
    //     validate_sequences_notes_trait(&mut s);
    // }
}
