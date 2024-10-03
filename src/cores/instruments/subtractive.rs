// Copyright (c) 2024 Mike Tsao

use crate::{cores::effects::BiQuadFilterLowPass24dbCore, prelude::*};
use anyhow::anyhow;
use core::fmt::Debug;
use delegate::delegate;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use strum_macros::{Display, EnumCount, EnumIter, FromRepr};

/// The source directory for the subtractive synth's patch files.
pub static PATCH_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/patches/subtractive");

/// Possible modulation targets.
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Display,
    EnumCount,
    EnumIter,
    FromRepr,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum LfoRouting {
    #[default]
    None,
    Amplitude,
    Pitch,
    PulseWidth,
    FilterCutoff,
    FilterResonance,
    Pitch2,
    PulseWidth2,
}

#[allow(missing_docs)]
#[derive(Debug, Default)]
pub struct SubtractiveSynthVoice {
    pub oscillator_1: Oscillator,
    pub oscillator_2: Oscillator,
    pub oscillator_2_sync: bool,
    pub oscillator_mix: Normal, // 1.0 = entirely osc 0, 0.0 = entirely osc 1.
    pub amp_envelope: Envelope,
    pub dca: Dca,

    pub lfo: Oscillator,
    pub lfo_routing: LfoRouting,
    pub lfo_depth: Normal,

    pub filter: BiQuadFilterLowPass24dbCore,
    pub filter_cutoff_start: Normal,
    pub filter_cutoff_end: Normal,
    pub filter_envelope: Envelope,

    note_on_key: u7,
    note_on_velocity: u7,
    steal_is_underway: bool,

    sample: StereoSample,

    amp_envelope_buffer: GenerationBuffer<Normal>,
    filter_envelope_buffer: GenerationBuffer<Normal>,
    lfo_buffer: GenerationBuffer<BipolarNormal>,
    mono_buffer: GenerationBuffer<Sample>,
}
impl IsStereoSampleVoice for SubtractiveSynthVoice {}
impl IsVoice<StereoSample> for SubtractiveSynthVoice {}
impl PlaysNotes for SubtractiveSynthVoice {
    fn is_playing(&self) -> bool {
        !self.amp_envelope.is_idle()
    }
    fn note_on(&mut self, key: u7, velocity: u7) {
        if self.is_playing() {
            self.steal_is_underway = true;
            self.note_on_key = key;
            self.note_on_velocity = velocity;
            self.amp_envelope.trigger_shutdown();
        } else {
            self.amp_envelope.trigger_attack();
            self.filter_envelope.trigger_attack();
            self.set_frequency_hz(MidiNote::from_repr(key.as_int() as usize).unwrap().into());
        }
    }
    fn aftertouch(&mut self, _velocity: u7) {
        // TODO: do something
    }
    fn note_off(&mut self, _velocity: u7) {
        self.amp_envelope.trigger_release();
        self.filter_envelope.trigger_release();
    }
}
impl Generates<StereoSample> for SubtractiveSynthVoice {
    fn generate(&mut self, values: &mut [StereoSample]) -> bool {
        let mut generated_signal = false;
        self.amp_envelope_buffer.resize(values.len());
        self.filter_envelope_buffer.resize(values.len());
        self.lfo_buffer.resize(values.len());
        self.mono_buffer.resize(values.len());
        let is_playing = self.is_playing();

        if is_playing {
            self.amp_envelope
                .generate(self.amp_envelope_buffer.buffer_mut());
            self.filter_envelope
                .generate(self.filter_envelope_buffer.buffer_mut());
            if matches!(self.lfo_routing, LfoRouting::None) {
                self.lfo_buffer.buffer_mut().fill(Default::default());
            } else {
                self.lfo.generate(self.lfo_buffer.buffer_mut());
            };
        }
        if self.steal_is_underway {
            self.steal_is_underway = false;
            self.note_on(self.note_on_key, self.note_on_velocity);
        }

        for (i, v) in self.mono_buffer.buffer_mut().iter_mut().enumerate() {
            *v = if is_playing {
                let mut osc_1_buffer = [BipolarNormal::default(); 1];
                let mut osc_2_buffer = [BipolarNormal::default(); 1];
                let lfo = self.lfo_buffer.buffer()[i];
                let amp_env_amplitude = self.amp_envelope_buffer.buffer()[i];
                let filter_env_amplitude = self.filter_envelope_buffer.buffer()[i];

                // LFO
                if matches!(self.lfo_routing, LfoRouting::Pitch) {
                    let lfo_for_pitch = lfo * self.lfo_depth;
                    self.oscillator_1.set_frequency_modulation(lfo_for_pitch);
                    self.oscillator_2.set_frequency_modulation(lfo_for_pitch);
                } else if matches!(self.lfo_routing, LfoRouting::Pitch2) {
                    let lfo_for_pitch = lfo * self.lfo_depth;
                    self.oscillator_2.set_frequency_modulation(lfo_for_pitch);
                } else if matches!(self.lfo_routing, LfoRouting::PulseWidth2) {
                    let lfo_for_pitch = lfo * self.lfo_depth;
                    self.oscillator_2
                        .set_waveform(Waveform::PulseWidth(lfo_for_pitch.into()));
                }

                // Oscillators
                if self.oscillator_2_sync && self.oscillator_1.should_sync() {
                    self.oscillator_2.sync();
                }
                self.oscillator_1.generate(&mut osc_1_buffer);
                self.oscillator_2.generate(&mut osc_2_buffer);

                let osc_sum = {
                    osc_1_buffer[0] * self.oscillator_mix
                        + osc_2_buffer[0] * (Normal::maximum() - self.oscillator_mix)
                };

                // Filters
                //
                // https://aempass.blogspot.com/2014/09/analog-and-welshs-synthesizer-cookbook.html
                if self.filter_cutoff_end != Normal::zero() {
                    let new_cutoff_percentage = self.filter_cutoff_start
                        + (1.0 - self.filter_cutoff_start)
                            * self.filter_cutoff_end
                            * filter_env_amplitude;
                    self.filter.set_cutoff(new_cutoff_percentage.into());
                } else if matches!(self.lfo_routing, LfoRouting::FilterCutoff) {
                    let lfo_for_cutoff = lfo * self.lfo_depth;
                    self.filter
                        .set_cutoff((self.filter_cutoff_start * (lfo_for_cutoff.0 + 1.0)).into());
                } else if matches!(self.lfo_routing, LfoRouting::FilterResonance) {
                    // TODO - it's unlikely this is correct. I copied/pasted
                    // while converting old patches and for the first time
                    // encountered a resonance setting.
                    let lfo_for_resonance = lfo * self.lfo_depth;
                    self.filter.set_passband_ripple(
                        (self.filter_cutoff_start * (lfo_for_resonance.0 + 1.0)).into(),
                    );
                }
                let filtered_mix = self.filter.transform_channel(0, Sample::from(osc_sum)).0;

                // LFO amplitude modulation
                let lfo_for_amplitude =
                    Normal::from(if matches!(self.lfo_routing, LfoRouting::Amplitude) {
                        lfo * self.lfo_depth
                    } else {
                        BipolarNormal::zero()
                    });

                // Final
                let sample = Sample(filtered_mix * amp_env_amplitude.0 * lfo_for_amplitude.0);
                generated_signal |= sample != Sample::default();
                sample
            } else {
                Sample::SILENCE
            };
        }

        // Make stereo from mono
        self.dca
            .transform_batch_to_stereo(self.mono_buffer.buffer(), values);

        generated_signal
    }
}
impl Serializable for SubtractiveSynthVoice {}
impl Configurable for SubtractiveSynthVoice {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.lfo.update_sample_rate(sample_rate);
        self.amp_envelope.update_sample_rate(sample_rate);
        self.filter_envelope.update_sample_rate(sample_rate);
        self.filter.update_sample_rate(sample_rate);
        self.oscillator_1.update_sample_rate(sample_rate);
        self.oscillator_2.update_sample_rate(sample_rate);
    }
}
impl SubtractiveSynthVoice {
    #[allow(missing_docs)]
    pub fn new_with(
        oscillator_1: &Oscillator,
        oscillator_2: &Oscillator,
        oscillator_2_sync: bool,
        oscillator_mix: Normal,
        amp_envelope: &Envelope,
        dca: &Dca,
        lfo: &Oscillator,
        lfo_routing: LfoRouting,
        lfo_depth: Normal,
        filter: &BiQuadFilterLowPass24dbCore,
        filter_cutoff_start: Normal,
        filter_cutoff_end: Normal,
        filter_envelope: &Envelope,
    ) -> Self {
        Self {
            oscillator_1: oscillator_1.make_another(),
            oscillator_2: oscillator_2.make_another(),
            oscillator_2_sync,
            oscillator_mix,
            amp_envelope: amp_envelope.make_another(),
            dca: dca.make_another(),
            lfo: lfo.make_another(),
            lfo_routing,
            lfo_depth,
            filter: filter.make_another(),
            filter_cutoff_start,
            filter_cutoff_end,
            filter_envelope: filter_envelope.make_another(),
            ..Default::default()
        }
    }

    fn set_frequency_hz(&mut self, frequency_hz: FrequencyHz) {
        // It's safe to set the frequency on a fixed-frequency oscillator; the
        // fixed frequency is stored separately and takes precedence.
        self.oscillator_1.set_frequency(frequency_hz);
        self.oscillator_2.set_frequency(frequency_hz);
    }

    #[allow(missing_docs)]
    pub fn set_lfo_depth(&mut self, lfo_depth: Normal) {
        self.lfo_depth = lfo_depth;
    }

    #[allow(missing_docs)]
    pub fn set_filter_cutoff_start(&mut self, filter_cutoff_start: Normal) {
        self.filter_cutoff_start = filter_cutoff_start;
    }

    #[allow(missing_docs)]
    pub fn set_filter_cutoff_end(&mut self, filter_cutoff_end: Normal) {
        self.filter_cutoff_end = filter_cutoff_end;
    }

    #[allow(missing_docs)]
    pub fn set_oscillator_2_sync(&mut self, oscillator_2_sync: bool) {
        self.oscillator_2_sync = oscillator_2_sync;
    }

    #[allow(missing_docs)]
    pub fn set_oscillator_mix(&mut self, oscillator_mix: Normal) {
        self.oscillator_mix = oscillator_mix;
    }

    #[allow(missing_docs)]
    pub fn amp_envelope_mut(&mut self) -> &mut Envelope {
        &mut self.amp_envelope
    }

    #[allow(missing_docs)]
    pub fn filter_mut(&mut self) -> &mut BiQuadFilterLowPass24dbCore {
        &mut self.filter
    }

    #[allow(missing_docs)]
    pub fn oscillator_2_sync(&self) -> bool {
        self.oscillator_2_sync
    }

    #[allow(missing_docs)]
    pub fn oscillator_mix(&self) -> Normal {
        self.oscillator_mix
    }

    #[allow(missing_docs)]
    pub fn lfo_routing(&self) -> LfoRouting {
        self.lfo_routing
    }

    #[allow(missing_docs)]
    pub fn lfo_depth(&self) -> Normal {
        self.lfo_depth
    }

    #[allow(missing_docs)]
    pub fn filter_cutoff_start(&self) -> Normal {
        self.filter_cutoff_start
    }

    #[allow(missing_docs)]
    pub fn filter_cutoff_end(&self) -> Normal {
        self.filter_cutoff_end
    }

    #[allow(missing_docs)]
    pub fn clone_fresh(&self) -> Self {
        todo!(" fill in the important fields... note that this likely matches the serde fields.");
        // let r = Self::default();
        // r
    }
}

/// A subtractive synthesizer inspired by Fred Welsh's [Welsh's Synthesizer
/// Cookbook](https://www.amazon.com/dp/B000ERHA4S/).
#[allow(missing_docs)]
#[derive(Debug, Default, Builder, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(default, build_fn(private, name = "build_from_builder"))]
pub struct SubtractiveSynthCore {
    pub preset_name: Option<String>,

    #[control]
    pub oscillator_1: Oscillator,
    #[control]
    pub oscillator_2: Oscillator,
    #[control]
    pub oscillator_2_sync: bool,
    #[control]
    pub oscillator_mix: Normal, // 1.0 = entirely osc 0, 0.0 = entirely osc 1.
    #[control]
    pub amp_envelope: Envelope,
    #[control]
    pub dca: Dca,

    #[control]
    pub lfo: Oscillator,
    pub lfo_routing: LfoRouting,
    #[control]
    pub lfo_depth: Normal,

    #[control]
    pub filter: BiQuadFilterLowPass24dbCore,
    #[control]
    pub filter_cutoff_start: Normal,
    #[control]
    pub filter_cutoff_end: Normal,
    #[control]
    pub filter_envelope: Envelope,

    #[serde(skip)]
    #[builder(setter(skip))]
    pub inner: Synthesizer<SubtractiveSynthVoice>,
}
impl SubtractiveSynthCoreBuilder {
    /// The overridden Builder build() method.
    pub fn build(&self) -> Result<SubtractiveSynthCore, SubtractiveSynthCoreBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }
}
impl SubtractiveSynthCore {
    const VOICE_CAPACITY: usize = 8;

    fn new_voice_store(&self) -> StealingVoiceStore<SubtractiveSynthVoice> {
        StealingVoiceStore::<SubtractiveSynthVoice>::new_with_voice(Self::VOICE_CAPACITY, || {
            SubtractiveSynthVoice::new_with(
                &self.oscillator_1,
                &self.oscillator_2,
                self.oscillator_2_sync,
                self.oscillator_mix,
                &self.amp_envelope,
                &self.dca,
                &self.lfo,
                self.lfo_routing,
                self.lfo_depth,
                &self.filter,
                self.filter_cutoff_start,
                self.filter_cutoff_end,
                &self.filter_envelope,
            )
        })
    }

    #[allow(missing_docs)]
    pub fn load_patch_from_json(json: &str) -> anyhow::Result<Self> {
        let mut patch = serde_json::from_str::<Self>(&json)?;
        patch.after_deser();
        Ok(patch)
    }

    #[allow(missing_docs)]
    pub fn load_patch(path: &PathBuf) -> anyhow::Result<Self> {
        let mut path = path.clone();
        path.set_extension("json");
        let json = std::fs::read_to_string(&path)?;
        Self::load_patch_from_json(json.as_str())
    }

    #[allow(missing_docs)]
    pub fn save_patch(&mut self, path: &PathBuf) -> anyhow::Result<()> {
        let mut path = path.clone();
        path.set_extension("json");
        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    #[allow(missing_docs)]
    pub fn load_internal_patch(name: &str) -> anyhow::Result<Self> {
        let path = format!("{}.json", name);
        if let Some(file) = PATCH_DIR.get_file(path) {
            if let Some(json) = file.contents_utf8() {
                Self::load_patch_from_json(json)
            } else {
                Err(anyhow!("Couldn't read patch named '{name}'"))
            }
        } else {
            Err(anyhow!("Couldn't find patch named '{name}'"))
        }
    }

    #[allow(missing_docs)]
    pub fn preset_name(&self) -> Option<&String> {
        self.preset_name.as_ref()
    }
}
impl Generates<StereoSample> for SubtractiveSynthCore {
    fn generate(&mut self, values: &mut [StereoSample]) -> bool {
        self.inner.generate(values)
    }
}
impl Serializable for SubtractiveSynthCore {
    fn before_ser(&mut self) {}

    fn after_deser(&mut self) {
        self.inner =
            Synthesizer::<SubtractiveSynthVoice>::new_with(Box::new(self.new_voice_store()));
    }
}
impl Configurable for SubtractiveSynthCore {
    delegate! {
        to self.inner {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl HandlesMidi for SubtractiveSynthCore {
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        midi_messages_fn: &mut MidiMessagesFn,
    ) {
        match message {
            #[allow(unused_variables)]
            MidiMessage::ProgramChange { program } => {
                todo!()
                // if let Some(program) = GeneralMidiProgram::from_u8(program.as_int()) {
                //     if let Ok(_preset) = SubtractiveSynth::general_midi_preset(&program) {
                //         //  self.preset = preset;
                //     } else {
                //         println!("unrecognized patch from MIDI program change: {}", &program);
                //     }
                // }
                // None
            }
            _ => self
                .inner
                .handle_midi_message(channel, message, midi_messages_fn),
        }
    }
}
impl SubtractiveSynthCore {
    #[allow(missing_docs)]
    pub fn notify_change_oscillator_1(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.oscillator_1.update_from_prototype(&self.oscillator_1);
        });
    }
    #[allow(missing_docs)]
    pub fn notify_change_oscillator_2(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.oscillator_2.update_from_prototype(&self.oscillator_2);
        });
    }
    #[allow(missing_docs)]
    pub fn notify_change_amp_envelope(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.amp_envelope.update_from_prototype(&self.amp_envelope);
        });
    }
    #[allow(missing_docs)]
    pub fn notify_change_dca(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.dca.update_from_prototype(&self.dca);
        });
    }
    #[allow(missing_docs)]
    pub fn notify_change_lfo(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.lfo.update_from_prototype(&self.lfo);
        });
    }
    #[allow(missing_docs)]
    pub fn notify_change_filter(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.filter.update_from_prototype(&self.filter);
        });
    }
    #[allow(missing_docs)]
    pub fn notify_change_filter_envelope(&mut self) {
        self.inner.voices_mut().for_each(|v| {
            v.filter_envelope
                .update_from_prototype(&self.filter_envelope);
        });
    }

    #[allow(missing_docs)]
    pub fn set_oscillator_2_sync(&mut self, oscillator_2_sync: bool) {
        self.oscillator_2_sync = oscillator_2_sync;
        self.inner
            .voices_mut()
            .for_each(|v| v.set_oscillator_2_sync(self.oscillator_2_sync));
    }

    #[allow(missing_docs)]
    pub fn set_oscillator_mix(&mut self, oscillator_mix: Normal) {
        self.oscillator_mix = oscillator_mix;
        self.inner
            .voices_mut()
            .for_each(|v| v.set_oscillator_mix(self.oscillator_mix));
    }

    #[allow(missing_docs)]
    pub fn set_lfo_depth(&mut self, lfo_depth: Normal) {
        self.lfo_depth = lfo_depth;
        self.inner
            .voices_mut()
            .for_each(|v| v.set_lfo_depth(self.lfo_depth));
    }

    #[allow(missing_docs)]
    pub fn set_filter_cutoff_start(&mut self, filter_cutoff_start: Normal) {
        self.filter_cutoff_start = filter_cutoff_start;
        self.inner
            .voices_mut()
            .for_each(|v| v.set_filter_cutoff_start(self.filter_cutoff_start));
    }

    #[allow(missing_docs)]
    pub fn set_filter_cutoff_end(&mut self, filter_cutoff_end: Normal) {
        self.filter_cutoff_end = filter_cutoff_end;
        self.inner
            .voices_mut()
            .for_each(|v| v.set_filter_cutoff_end(self.filter_cutoff_end));
    }
}
