// Copyright (c) 2024 Mike Tsao

use crate::{prelude::*, traits::GenerationBuffer};
use anyhow::{anyhow, Result};
use derivative::Derivative;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use synonym::Synonym;

/// Newtype for the number of voices in a multi-voice instrument.
#[derive(Synonym, Derivative, Deserialize, Serialize)]
#[derivative(Default)]
#[synonym(skip(Default))]
pub struct VoiceCount(#[derivative(Default(value = "8"))] pub usize);

/// A [StoresVoices](crate::traits::StoresVoices) that fails when too many
/// voices are used simultaneously.
#[derive(Debug, Default)]
pub struct VoiceStore<V: IsStereoSampleVoice> {
    sample_rate: SampleRate,
    voices: Vec<Box<V>>,
    notes_playing: Vec<u7>,
    voice_buffer: GenerationBuffer<StereoSample>,
}
impl<V: IsStereoSampleVoice> StoresVoices for VoiceStore<V> {
    type Voice = V;

    fn voice_count(&self) -> usize {
        self.voices.len()
    }

    fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_playing()).count()
    }

    fn get_voice(&mut self, key: &u7) -> Result<&mut Box<Self::Voice>> {
        // If we have a voice already going for this note, return it.
        if let Some(index) = self.notes_playing.iter().position(|note| *key == *note) {
            return Ok(&mut self.voices[index]);
        }
        // If we can find an inactive voice, return it.
        for (index, voice) in self.voices.iter().enumerate() {
            if voice.is_playing() {
                continue;
            }
            self.notes_playing[index] = *key;
            return Ok(&mut self.voices[index]);
        }

        Err(anyhow!("out of voices"))
    }

    fn voices<'a>(&'a self) -> Box<dyn Iterator<Item = &Box<Self::Voice>> + 'a> {
        Box::new(self.voices.iter())
    }

    fn voices_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Box<Self::Voice>> + 'a> {
        Box::new(self.voices.iter_mut())
    }
}
impl<V: IsStereoSampleVoice> Generates<StereoSample> for VoiceStore<V> {
    fn generate(&mut self, values: &mut [StereoSample]) -> bool {
        let mut generated_signal = false;
        self.voice_buffer.resize(values.len());
        self.voices.iter_mut().for_each(|v| {
            generated_signal |= v.generate(self.voice_buffer.buffer_mut());
            values
                .iter_mut()
                .zip(self.voice_buffer.buffer().iter())
                .for_each(|(d, s)| *d += *s);
        });
        self.voices.iter().enumerate().for_each(|(index, voice)| {
            if !voice.is_playing() {
                self.notes_playing[index] = u7::from(0);
            }
        });
        generated_signal
    }
}
impl<V: IsStereoSampleVoice> Configurable for VoiceStore<V> {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
        self.voices
            .iter_mut()
            .for_each(|v| v.update_sample_rate(sample_rate));
    }
}
impl<V: IsStereoSampleVoice> VoiceStore<V> {
    fn new() -> Self {
        Self {
            sample_rate: Default::default(),
            voices: Default::default(),
            notes_playing: Default::default(),
            voice_buffer: Default::default(),
        }
    }
    fn add_voice(&mut self, voice: Box<V>) {
        self.voices.push(voice);
        self.notes_playing.push(u7::from(0));
    }

    #[allow(missing_docs)]
    pub fn new_with_voice<F>(voice_capacity: VoiceCount, new_voice_fn: F) -> Self
    where
        F: Fn() -> V,
    {
        let mut voice_store = Self::new();
        for _ in 0..voice_capacity.0 {
            voice_store.add_voice(Box::new(new_voice_fn()));
        }
        voice_store
    }
}

/// A [StoresVoices](crate::traits::StoresVoices) that steals voices as needed.
#[derive(Debug)]
pub struct StealingVoiceStore<V: IsStereoSampleVoice> {
    sample_rate: SampleRate,
    voices: Vec<Box<V>>,
    notes_playing: Vec<u7>,
    voice_buffer: GenerationBuffer<StereoSample>,
}
impl<V: IsStereoSampleVoice> StoresVoices for StealingVoiceStore<V> {
    type Voice = V;

    fn voice_count(&self) -> usize {
        self.voices.len()
    }

    fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_playing()).count()
    }

    fn get_voice(&mut self, key: &u7) -> Result<&mut Box<Self::Voice>> {
        // If we have a voice already going for this note, return it.
        if let Some(index) = self.notes_playing.iter().position(|note| *key == *note) {
            return Ok(&mut self.voices[index]);
        }
        // If we can find an inactive voice, return it.
        for (index, voice) in self.voices.iter().enumerate() {
            if voice.is_playing() {
                continue;
            }
            self.notes_playing[index] = *key;
            return Ok(&mut self.voices[index]);
        }

        // We need to steal a voice. For now, let's just pick the first one in
        // the list.
        let index = 0;
        self.notes_playing[index] = *key;
        return Ok(&mut self.voices[index]);

        #[allow(unreachable_code)]
        Err(anyhow!("out of voices"))
    }

    fn voices<'a>(&'a self) -> Box<dyn Iterator<Item = &Box<Self::Voice>> + 'a> {
        Box::new(self.voices.iter())
    }

    fn voices_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Box<Self::Voice>> + 'a> {
        Box::new(self.voices.iter_mut())
    }
}
impl<V: IsStereoSampleVoice> Generates<StereoSample> for StealingVoiceStore<V> {
    fn generate(&mut self, values: &mut [StereoSample]) -> bool {
        let mut generated_signal = false;
        self.voice_buffer.resize(values.len());
        self.voices.iter_mut().for_each(|v| {
            generated_signal |= v.generate(self.voice_buffer.buffer_mut());
            values
                .iter_mut()
                .zip(self.voice_buffer.buffer().iter())
                .for_each(|(d, s)| *d += *s);
        });
        self.voices.iter().enumerate().for_each(|(index, voice)| {
            if !voice.is_playing() {
                self.notes_playing[index] = u7::from(0);
            }
        });
        generated_signal
    }
}
impl<V: IsStereoSampleVoice> Configurable for StealingVoiceStore<V> {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
        self.voices
            .iter_mut()
            .for_each(|v| v.update_sample_rate(sample_rate));
    }
}
impl<V: IsStereoSampleVoice> StealingVoiceStore<V> {
    fn new() -> Self {
        Self {
            sample_rate: Default::default(),
            voices: Default::default(),
            notes_playing: Default::default(),
            voice_buffer: Default::default(),
        }
    }

    #[allow(missing_docs)]
    pub fn new_with_voice<F>(voice_capacity: usize, new_voice_fn: F) -> Self
    where
        F: Fn() -> V,
    {
        let mut voice_store = Self::new();
        for _ in 0..voice_capacity {
            voice_store.add_voice(Box::new(new_voice_fn()));
        }
        voice_store
    }

    fn add_voice(&mut self, voice: Box<V>) {
        self.voices.push(voice);
        self.notes_playing.push(u7::from(0));
    }
}

/// A [StoresVoices](crate::traits::StoresVoices) that assumes a specific voice
/// is dedicated to each note. A good example is a drumkit sampler, which uses
/// the same [IsVoice](crate::traits::IsVoice) whenever a particular sample is
/// played.
#[derive(Debug)]
pub struct VoicePerNoteStore<V: IsStereoSampleVoice> {
    sample_rate: SampleRate,
    voices: FxHashMap<u7, Box<V>>,
    voice_buffer: GenerationBuffer<StereoSample>,
}
impl<V: IsStereoSampleVoice> StoresVoices for VoicePerNoteStore<V> {
    type Voice = V;

    fn voice_count(&self) -> usize {
        self.voices.len()
    }

    fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|(_k, v)| v.is_playing()).count()
    }

    fn get_voice(&mut self, key: &u7) -> Result<&mut Box<Self::Voice>> {
        if let Some(voice) = self.voices.get_mut(key) {
            return Ok(voice);
        }
        Err(anyhow!("no voice for key {}", key))
    }

    fn voices<'a>(&'a self) -> Box<dyn Iterator<Item = &Box<Self::Voice>> + 'a> {
        Box::new(self.voices.values())
    }

    fn voices_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Box<Self::Voice>> + 'a> {
        let values = self.voices.values_mut();
        Box::new(values)
    }
}
impl<V: IsStereoSampleVoice> Generates<StereoSample> for VoicePerNoteStore<V> {
    fn generate(&mut self, values: &mut [StereoSample]) -> bool {
        let mut generated_signal = false;
        self.voice_buffer.resize(values.len());
        self.voices.values_mut().for_each(|v| {
            generated_signal |= v.generate(self.voice_buffer.buffer_mut());
            values
                .iter_mut()
                .zip(self.voice_buffer.buffer().iter())
                .for_each(|(d, s)| *d += *s);
        });
        generated_signal
    }
}
impl<V: IsStereoSampleVoice> Configurable for VoicePerNoteStore<V> {
    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
        self.voices
            .values_mut()
            .for_each(|v| v.update_sample_rate(sample_rate));
    }
}
impl<V: IsStereoSampleVoice> Default for VoicePerNoteStore<V> {
    fn default() -> Self {
        Self::new()
    }
}
#[allow(missing_docs)]
impl<V: IsStereoSampleVoice> VoicePerNoteStore<V> {
    pub fn new() -> Self {
        Self {
            sample_rate: Default::default(),
            voices: Default::default(),
            voice_buffer: Default::default(),
        }
    }

    pub fn new_with_voices(voice_iter: impl Iterator<Item = (u7, V)>) -> Self {
        let mut voice_store = Self::new();
        for (key, voice) in voice_iter {
            voice_store.add_voice(key, Box::new(voice));
        }
        voice_store
    }

    pub fn add_voice(&mut self, key: u7, voice: Box<V>) {
        self.voices.insert(key, voice);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::elements::OscillatorBuilder;

    use super::*;
    use float_cmp::approx_eq;
    use more_asserts::assert_gt;

    #[derive(Debug)]
    pub struct TestVoice {
        sample_rate: SampleRate,
        oscillator: Oscillator,
        envelope: Envelope,

        note_on_key: u7,
        note_on_velocity: u7,
        steal_is_underway: bool,

        osc_buffer: GenerationBuffer<BipolarNormal>,
        env_buffer: GenerationBuffer<Normal>,
    }
    impl IsStereoSampleVoice for TestVoice {}
    impl IsVoice<StereoSample> for TestVoice {}
    impl PlaysNotes for TestVoice {
        fn is_playing(&self) -> bool {
            !self.envelope.is_idle()
        }

        fn note_on(&mut self, key: u7, velocity: u7) {
            if self.is_playing() {
                self.steal_is_underway = true;
                self.note_on_key = key;
                self.note_on_velocity = velocity;
                self.envelope.trigger_shutdown();
            } else {
                self.set_frequency_hz(key.into());
                self.envelope.trigger_attack();
            }
        }

        fn aftertouch(&mut self, _velocity: u7) {
            todo!()
        }

        fn note_off(&mut self, _velocity: u7) {
            self.envelope.trigger_release();
        }
    }
    impl Generates<StereoSample> for TestVoice {
        fn generate(&mut self, values: &mut [StereoSample]) -> bool {
            let mut generated_signal = false;
            if self.is_playing() {
                self.osc_buffer.resize(values.len());
                self.oscillator.generate(self.osc_buffer.buffer_mut());
                self.env_buffer.resize(values.len());
                self.envelope.generate(self.env_buffer.buffer_mut());
                if !self.is_playing() && self.steal_is_underway {
                    self.steal_is_underway = false;
                    self.note_on(self.note_on_key, self.note_on_velocity);
                }
                values
                    .iter_mut()
                    .zip(
                        self.osc_buffer
                            .buffer()
                            .iter()
                            .zip(self.env_buffer.buffer().iter()),
                    )
                    .for_each(|(dst, (osc, env))| {
                        let sample: StereoSample = (*osc * *env).into();
                        generated_signal |= sample != StereoSample::default();
                        *dst = sample;
                    });
            } else {
                values.fill(StereoSample::SILENCE);
            }
            generated_signal
        }
    }
    impl Configurable for TestVoice {
        fn sample_rate(&self) -> SampleRate {
            self.sample_rate
        }

        fn update_sample_rate(&mut self, sample_rate: SampleRate) {
            self.sample_rate = sample_rate;
            self.oscillator.update_sample_rate(sample_rate);
            self.envelope.update_sample_rate(sample_rate);
        }
    }
    impl TestVoice {
        pub(crate) fn new() -> Self {
            Self {
                sample_rate: Default::default(),
                oscillator: OscillatorBuilder::default().build().unwrap(),
                envelope: EnvelopeBuilder::default()
                    .attack(Normal::minimum())
                    .decay(Normal::minimum())
                    .sustain(Normal::maximum())
                    .release(Normal::minimum())
                    .build()
                    .unwrap(),
                note_on_key: Default::default(),
                note_on_velocity: Default::default(),
                steal_is_underway: Default::default(),
                osc_buffer: Default::default(),
                env_buffer: Default::default(),
            }
        }
        fn set_frequency_hz(&mut self, frequency_hz: FrequencyHz) {
            self.oscillator.set_frequency(frequency_hz);
        }

        pub fn debug_is_shutting_down(&self) -> bool {
            self.envelope.debug_is_shutting_down()
        }

        pub fn debug_oscillator_frequency(&self) -> FrequencyHz {
            self.oscillator.frequency()
        }
    }

    #[test]
    fn simple_voice_store_mainline() {
        let mut voice_store =
            VoiceStore::<TestVoice>::new_with_voice(VoiceCount(2), || TestVoice::new());
        assert_gt!(!voice_store.voice_count(), 0);
        assert_eq!(voice_store.active_voice_count(), 0);

        let mut buffer = [StereoSample::default(); 1];

        // Request and start the maximum number of voices.
        if let Ok(voice) = voice_store.get_voice(&u7::from(60)) {
            assert!(!voice.is_playing());
            voice.note_on(u7::from(60), u7::from(127));
            voice.generate(&mut buffer); // We must tick() to register the trigger.
            assert!(voice.is_playing());
        }
        if let Ok(voice) = voice_store.get_voice(&u7::from(61)) {
            voice.note_on(u7::from(61), u7::from(127));
            voice.generate(&mut buffer);
        }

        // Request a voice for a new note that would exceed the count. Should
        // fail.
        assert!(voice_store.get_voice(&u7::from(62)).is_err());

        // Request to get back a voice that's already playing.
        if let Ok(voice) = voice_store.get_voice(&u7::from(60)) {
            assert!(voice.is_playing());
            voice.note_off(u7::from(127));

            // All TestVoice envelope times are instantaneous, so we know the
            // release completes after asking for the next sample.
            voice.generate(&mut buffer);
            assert!(!voice.is_playing());
        }
    }

    #[test]
    fn stealing_voice_store_mainline() {
        let mut voice_store =
            StealingVoiceStore::<TestVoice>::new_with_voice(2, || TestVoice::new());
        assert_gt!(voice_store.voice_count(), 0);
        assert_eq!(voice_store.active_voice_count(), 0);

        let mut buffer = [StereoSample::default(); 1];

        // Request and start the full number of voices.
        if let Ok(voice) = voice_store.get_voice(&u7::from(60)) {
            assert!(!voice.is_playing());
            voice.note_on(u7::from(60), u7::from(127));
            voice.generate(&mut buffer); // We must tick() to register the trigger.
            assert!(voice.is_playing());
        }
        if let Ok(voice) = voice_store.get_voice(&u7::from(61)) {
            assert!(!voice.is_playing());
            voice.note_on(u7::from(61), u7::from(127));
            voice.generate(&mut buffer);
            assert!(voice.is_playing());
        }

        // Request a voice for a new note that would exceed the count. It should
        // already be playing, because we're about to steal it.
        if let Ok(voice) = voice_store.get_voice(&u7::from(62)) {
            assert!(voice.is_playing());

            // This is testing the shutdown state, rather than the voice store,
            // but I'm feeling lazy today.
            voice.note_on(u7::from(62), u7::from(127));
            voice.generate(&mut buffer);
            assert!(voice.debug_is_shutting_down());
        } else {
            assert!(false, "StealingVoiceStore didn't return a voice");
        }
    }

    #[test]
    fn voice_store_simultaneous_events() {
        let mut voice_store =
            VoiceStore::<TestVoice>::new_with_voice(VoiceCount(2), || TestVoice::new());
        assert_gt!(voice_store.voice_count(), 0);
        assert_eq!(voice_store.active_voice_count(), 0);

        // Request multiple voices during the same tick.
        if let Ok(voice) = voice_store.get_voice(&u7::from(60)) {
            voice.note_on(u7::from(60), u7::from(127));
            assert!(
                voice.is_playing(),
                "New voice should be marked is_playing() immediately after attack()"
            );
        }
        if let Ok(voice) = voice_store.get_voice(&u7::from(61)) {
            voice.note_on(u7::from(62), u7::from(127));
            assert!(
                voice.is_playing(),
                "New voice should be marked is_playing() immediately after attack()"
            );
        }

        let mut buffer = [StereoSample::default(); 1];
        let _ = voice_store.generate(&mut buffer);
        assert_eq!(voice_store.active_voice_count(), 2, "voices with pending attacks() should have been handled, and they should now be is_playing()");

        // Now ask for both voices again. Each should be playing and each should
        // have its individual frequency.
        if let Ok(voice) = voice_store.get_voice(&u7::from(60)) {
            assert!(voice.is_playing());
            assert!(
                approx_eq!(
                    ParameterType,
                    voice.debug_oscillator_frequency().0,
                    FrequencyHz::from(MidiNote::C4).0
                ),
                "we should have gotten back the same voice for the requested note"
            );
        }
        if let Ok(voice) = voice_store.get_voice(&u7::from(61)) {
            assert!(voice.is_playing());
            assert!(
                approx_eq!(
                    ParameterType,
                    voice.debug_oscillator_frequency().0,
                    FrequencyHz::from(MidiNote::D4).0
                ),
                "we should have gotten back the same voice for the requested note"
            );
        }
        let _ = voice_store.generate(&mut buffer);

        // Finally, mark a note done and then ask for a new one. We should get
        // assigned the one we just gave up.
        //
        // Note that we're taking advantage of the fact that TestVoice has
        // instantaneous envelope parameters, which means we can treat the
        // release as the same as the note stopping playing. For most voices
        // with nonzero release, we'd have to wait more time for the voice to
        // stop on its own. This is also why we need to spin the source_audio()
        // loop in between the two get_voice() requests; it's actually correct
        // for the system to consider a voice to still be playing after
        // release() during the same tick.
        if let Ok(voice) = voice_store.get_voice(&u7::from(60)) {
            assert!(
                approx_eq!(
                    ParameterType,
                    voice.debug_oscillator_frequency().0,
                    FrequencyHz::from(MidiNote::C4).0
                ),
                "we should have gotten back the same voice for the requested note"
            );
            voice.note_off(u7::from(127));
        }
        let _ = voice_store.generate(&mut buffer);
        if let Ok(voice) = voice_store.get_voice(&u7::from(62)) {
            // This is a bit too cute. We assume that we're getting back the
            // voice that serviced note #60 because (1) we set up the voice
            // store with only two voices, and the other one is busy, and (2) we
            // happen to know that this voice store recycles voices rather than
            // instantiating new ones. (2) is very likely to remain true for all
            // voice stores, but it's a little loosey-goosey right now.
            assert!(
                approx_eq!(
                    ParameterType,
                    voice.debug_oscillator_frequency().0,
                    FrequencyHz::from(MidiNote::C4).0 // 60, not 62!!
                ),
                "we should have gotten the defunct voice for a new note"
            );
        } else {
            panic!("ran out of notes unexpectedly");
        }
    }
}
