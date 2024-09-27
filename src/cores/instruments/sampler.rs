// Copyright (c) 2024 Mike Tsao

use crate::{
    prelude::*,
    util::{
        Paths, {SampleLibrary, SampleSource},
    },
};
use anyhow::{anyhow, Result};
use ensnare_proc_macros::Control;
use hound::WavReader;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, path::Path, sync::Arc};

/// One sampler voice. Combine multiple of these to make a sampling synth.
#[derive(Debug, Default)]
pub struct SamplerVoice {
    sample_rate: SampleRate,
    samples: Option<Arc<Vec<StereoSample>>>,

    root_frequency: FrequencyHz,
    frequency: FrequencyHz,

    was_reset: bool,
    is_playing: bool,
    sample_pointer: ParameterType,
    sample_pointer_delta: ParameterType,
}
impl IsVoice<StereoSample> for SamplerVoice {}
impl IsStereoSampleVoice for SamplerVoice {}
impl PlaysNotes for SamplerVoice {
    fn is_playing(&self) -> bool {
        self.is_playing
    }

    #[allow(unused_variables)]
    fn note_on(&mut self, key: u7, velocity: u7) {
        self.is_playing = true;
        self.sample_pointer = 0.0;
        self.frequency = MidiNote::from_repr(key.as_int() as usize).unwrap().into();
        self.sample_pointer_delta = (self.frequency / self.root_frequency).into();
    }

    #[allow(unused_variables)]
    fn aftertouch(&mut self, velocity: u7) {
        todo!()
    }

    #[allow(unused_variables)]
    fn note_off(&mut self, velocity: u7) {
        self.is_playing = false;
        self.sample_pointer = 0.0;
    }
}
impl Generates<StereoSample> for SamplerVoice {
    fn generate(&mut self, values: &mut [StereoSample]) -> bool {
        let mut generated_signal = false;

        for value in values {
            *value = {
                if self.is_playing {
                    if let Some(samples) = self.samples.as_ref() {
                        if samples.len() != 0 {
                            generated_signal = true;
                            samples[self.sample_pointer as usize]
                        } else {
                            StereoSample::SILENCE
                        }
                    } else {
                        StereoSample::SILENCE
                    }
                } else {
                    StereoSample::SILENCE
                }
            };

            if self.is_playing {
                if !self.was_reset {
                    self.sample_pointer += self.sample_pointer_delta;
                }
                if let Some(samples) = self.samples.as_ref() {
                    debug_assert_ne!(samples.len(), 0);
                    while self.sample_pointer as usize >= samples.len() {
                        self.is_playing = false;
                        self.sample_pointer -= samples.len() as f64;
                    }
                }
            }
            if self.was_reset {
                self.was_reset = false;
            }
        }
        generated_signal
    }
}
impl Serializable for SamplerVoice {}
impl Configurable for SamplerVoice {
    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
        self.was_reset = true;
    }
}
#[allow(missing_docs)]
impl SamplerVoice {
    pub fn new_with_samples(samples: Arc<Vec<StereoSample>>, root_frequency: FrequencyHz) -> Self {
        if !root_frequency.0.is_normal() {
            panic!("strange number given for root frequency: {root_frequency}");
        }
        let samples = if samples.len() != 0 {
            Some(samples)
        } else {
            None
        };
        Self {
            sample_rate: Default::default(),
            samples,
            root_frequency,
            frequency: Default::default(),
            was_reset: true,
            is_playing: Default::default(),
            sample_pointer: Default::default(),
            sample_pointer_delta: Default::default(),
        }
    }

    pub fn set_root_frequency(&mut self, root_frequency: FrequencyHz) {
        self.root_frequency = root_frequency;
    }
}

/// A sampling synthesizer.
#[derive(Debug, Control, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SamplerCore {
    source: SampleSource,

    #[control]
    root: FrequencyHz,

    #[serde(skip)]
    e: SamplerEphemerals,
}
#[derive(Debug, Default)]
pub struct SamplerEphemerals {
    calculated_root: FrequencyHz,

    inner: Synthesizer<SamplerVoice>,

    c: Configurables,
}
impl HandlesMidi for SamplerCore {
    fn handle_midi_message(
        &mut self,
        channel: MidiChannel,
        message: MidiMessage,
        midi_messages_fn: &mut MidiMessagesFn,
    ) {
        self.e
            .inner
            .handle_midi_message(channel, message, midi_messages_fn)
    }
}
impl Generates<StereoSample> for SamplerCore {
    fn generate(&mut self, values: &mut [StereoSample]) -> bool {
        self.e.inner.generate(values)
    }
}
impl Serializable for SamplerCore {}
impl Configurable for SamplerCore {
    fn sample_rate(&self) -> SampleRate {
        self.e.inner.sample_rate()
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.e.inner.update_sample_rate(sample_rate)
    }

    fn tempo(&self) -> Tempo {
        self.e.c.tempo()
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.e.c.update_tempo(tempo)
    }

    fn time_signature(&self) -> TimeSignature {
        self.e.c.time_signature()
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.e.c.update_time_signature(time_signature)
    }
}
#[allow(missing_docs)]
impl SamplerCore {
    pub fn load(&mut self) -> anyhow::Result<()> {
        let path = match &self.source {
            SampleSource::SampleLibrary(index) => {
                if let Some(path) = SampleLibrary::global().path(*index) {
                    Paths::global().build_sample(&Vec::default(), Path::new(&path))
                } else {
                    return Err(anyhow!("Couldn't find sample {index} in library"));
                }
            }
            SampleSource::Path(path_buf) => {
                Paths::global().build_sample(&Vec::default(), Path::new(&path_buf))
            }
        };
        let file = Paths::global().search_and_open(path.as_path())?;
        let samples = Self::read_samples_from_file(&file)?;
        let samples = Arc::new(samples);

        self.e.calculated_root = if self.root.0 > 0.0 {
            self.root
        } else
        // if let Ok(embedded_root_note) = Self::read_riff_metadata(&mut f2) {
        //  FrequencyHz::from(u7::from(embedded_root_note))
        //} else
        {
            FrequencyHz::from(440.0)
        };

        self.e.inner = Synthesizer::<SamplerVoice>::new_with(Box::new(
            VoiceStore::<SamplerVoice>::new_with_voice(VoiceCount::from(8), || {
                SamplerVoice::new_with_samples(Arc::clone(&samples), self.e.calculated_root)
            }),
        ));

        Ok(())
    }

    pub fn new_with(source: SampleSource, root: Option<FrequencyHz>) -> Self {
        let samples = Arc::new(Vec::default());
        let calculated_root = root.unwrap_or_default();
        let e = SamplerEphemerals {
            inner: Synthesizer::<SamplerVoice>::new_with(Box::new(
                VoiceStore::<SamplerVoice>::new_with_voice(VoiceCount::from(8), || {
                    SamplerVoice::new_with_samples(Arc::clone(&samples), calculated_root)
                }),
            )),
            calculated_root,
            ..Default::default()
        };

        Self {
            e,
            source,
            root: calculated_root,
        }
    }

    // https://forums.cockos.com/showthread.php?t=227118
    //
    // ** The acid chunk goes a little something like this:
    // **
    // ** 4 bytes          'acid'
    // ** 4 bytes (int)     length of chunk starting at next byte
    // **
    // ** 4 bytes (int)     type of file:
    // **        this appears to be a bit mask,however some combinations
    // **        are probably impossible and/or qualified as "errors"
    // **
    // **        0x01 On: One Shot         Off: Loop
    // **        0x02 On: Root note is Set Off: No root
    // **        0x04 On: Stretch is On,   Off: Strech is OFF
    // **        0x08 On: Disk Based       Off: Ram based
    // **        0x10 On: ??????????       Off: ????????? (Acidizer puts that ON)
    // **
    // ** 2 bytes (short)      root note
    // **        if type 0x10 is OFF : [C,C#,(...),B] -> [0x30 to 0x3B]
    // **        if type 0x10 is ON  : [C,C#,(...),B] -> [0x3C to 0x47]
    // **         (both types fit on same MIDI pitch albeit different octaves, so who cares)
    // **
    // ** 2 bytes (short)      ??? always set to 0x8000
    // ** 4 bytes (float)      ??? seems to be always 0
    // ** 4 bytes (int)        number of beats
    // ** 2 bytes (short)      meter denominator   //always 4 in SF/ACID
    // ** 2 bytes (short)      meter numerator     //always 4 in SF/ACID
    // **                      //are we sure about the order?? usually its num/denom
    // ** 4 bytes (float)      tempo
    // **
    #[allow(dead_code)]
    fn read_riff_metadata(_file: &mut File) -> Result<u8> {
        Err(anyhow!("riff_io crate is excluded"))
        // let riff = riff_io::RiffFile::open_with_file_handle(file)?;
        // let entries = riff.read_entries()?;
        // for entry in entries {
        //     match entry {
        //         riff_io::Entry::Chunk(chunk) => {
        //             // looking for chunk_id 'acid'
        //             if chunk.chunk_id == [97, 99, 105, 100] {
        //                 file.seek(std::io::SeekFrom::Start(chunk.data_offset as u64))?;
        //                 let mut bytes = Vec::default();
        //                 bytes.resize(chunk.data_size, 0);
        //                 let _ = file.read(&mut bytes)?;

        //                 let root_note_set = bytes[0] & 0x02 != 0;
        //                 let pitch_b = bytes[0] & 0x10 != 0;

        //                 if root_note_set {
        //                     // TODO: find a real WAV that has the pitch_b flag set
        //                     let root_note = bytes[4] - if pitch_b { 12 } else { 0 };
        //                     return Ok(root_note);
        //                 }
        //             }
        //         }
        //         _ => {}
        //     }
        // }
        // Err(anyhow!("Couldn't find root note in acid RIFF chunk"))
    }

    fn read_samples<T>(
        reader: &mut WavReader<BufReader<&File>>,
        channels: u16,
        scale_factor: SampleType,
    ) -> anyhow::Result<Vec<StereoSample>>
    where
        Sample: From<T>,
        T: hound::Sample,
    {
        let mut samples = Vec::default();
        if channels == 1 {
            for sample in reader.samples::<T>().flatten() {
                samples.push(StereoSample::from(Sample::from(sample) / scale_factor));
            }
        } else {
            debug_assert_eq!(channels, 2);
            loop {
                let mut iter = reader.samples::<T>();
                let left = iter.next();
                if let Some(Ok(left)) = left {
                    let right = iter.next();
                    if let Some(Ok(right)) = right {
                        let left = Sample::from(left) / scale_factor;
                        let right = Sample::from(right) / scale_factor;
                        samples.push(StereoSample(left, right));
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        Ok(samples)
    }

    pub fn read_samples_from_file(file: &File) -> anyhow::Result<Vec<StereoSample>> {
        let mut reader = hound::WavReader::new(BufReader::new(file))?;
        let spec = reader.spec();
        let itype_max: SampleType = 2.0f64.powi(spec.bits_per_sample as i32 - 1);

        match spec.sample_format {
            hound::SampleFormat::Float => {
                Self::read_samples::<f32>(&mut reader, spec.channels, itype_max)
            }
            hound::SampleFormat::Int => {
                Self::read_samples::<i32>(&mut reader, spec.channels, itype_max)
            }
        }
    }

    pub fn root(&self) -> FrequencyHz {
        self.root
    }

    pub fn set_root(&mut self, root: FrequencyHz) {
        self.root = root;
        self.e
            .inner
            .voices_mut()
            .for_each(|v| v.set_root_frequency(root));
    }

    pub fn calculated_root(&self) -> FrequencyHz {
        self.e.calculated_root
    }

    pub fn set_calculated_root(&mut self, calculated_root: FrequencyHz) {
        self.e.calculated_root = calculated_root;
    }

    pub fn source(&self) -> &SampleSource {
        &self.source
    }

    pub fn set_source(&mut self, source: SampleSource) {
        if self.source != source {
            self.source = source;
            let _ = self.load();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::FileType;
    use std::path::PathBuf;

    fn paths_with_test_data_dir() -> Paths {
        let mut paths = Paths::default();
        paths.push_hive(Paths::test_data_rel());
        paths
    }

    #[test]
    fn loading() {
        Paths::set_instance(paths_with_test_data_dir());
        let mut sampler =
            SamplerCore::new_with(SampleSource::Path("stereo-pluck.wav".into()), None);
        assert!(sampler.load().is_ok());
        assert_eq!(sampler.calculated_root(), FrequencyHz::from(440.0));
    }

    #[test]
    #[ignore = "Re-enable when SamplerParams knows how to handle String"]
    fn reading_acidized_metadata() {
        let filename = PathBuf::from("riff-acidized.wav");
        let mut file = std::fs::File::open(filename).unwrap();
        let root_note = SamplerCore::read_riff_metadata(&mut file);
        assert!(root_note.is_ok());
        assert_eq!(root_note.unwrap(), 57);

        let filename = PathBuf::from("riff-not-acidized.wav");
        let mut file = std::fs::File::open(filename).unwrap();
        let root_note = SamplerCore::read_riff_metadata(&mut file);
        assert!(root_note.is_err());
    }

    //    #[test]
    #[allow(dead_code)]
    fn reading_smpl_metadata() {
        let filename = PathBuf::from("riff-with-smpl.wav");
        let mut file = std::fs::File::open(filename).unwrap();
        let root_note = SamplerCore::read_riff_metadata(&mut file);
        assert!(root_note.is_ok());
        assert_eq!(root_note.unwrap(), 255);
    }

    #[test]
    #[ignore = "riff_io crate is disabled, so we can't read root frequencies from files"]
    fn loading_with_root_frequency() {
        Paths::set_instance(paths_with_test_data_dir());
        let mut sampler =
            SamplerCore::new_with(SampleSource::Path("riff-acidized.wav".into()), None);
        assert!(sampler.load().is_ok());
        eprintln!("calculated {} ", sampler.calculated_root());
        assert_eq!(
            sampler.calculated_root(),
            MidiNote::A3.into(),
            "acidized WAV should produce sample with embedded root note"
        );

        let mut sampler = SamplerCore::new_with(
            SampleSource::Path("riff-acidized.wav".into()),
            Some(123.0.into()),
        );
        assert!(sampler.load().is_ok());
        assert_eq!(
            sampler.calculated_root(),
            FrequencyHz::from(123.0),
            "specified parameter should override acidized WAV's embedded root note"
        );

        let mut sampler = SamplerCore::new_with(
            SampleSource::Path("riff-not-acidized.wav".into()),
            Some(123.0.into()),
        );
        assert!(sampler.load().is_ok());
        assert_eq!(
            sampler.calculated_root(),
            FrequencyHz::from(123.0),
            "specified parameter should be used for non-acidized WAV"
        );

        let mut sampler =
            SamplerCore::new_with(SampleSource::Path("riff-not-acidized.wav".into()), None);
        assert!(sampler.load().is_ok());
        assert_eq!(
            sampler.calculated_root(),
            MidiNote::A4.into(),
            "If there is neither an acidized WAV nor a provided frequency, sample should have root note A4 (440Hz)"
        );
    }

    #[test]
    fn sampler_makes_any_sound_at_all() {
        let paths = paths_with_test_data_dir();
        let file = paths.search_and_open_with_file_type(
            FileType::Sample,
            Path::new("square-440Hz-1-second-mono-24-bit-PCM.wav"),
        );
        assert!(file.is_ok());
        let samples = SamplerCore::read_samples_from_file(&file.unwrap());
        assert!(samples.is_ok());
        let samples = samples.unwrap();
        let mut voice = SamplerVoice::new_with_samples(Arc::new(samples), FrequencyHz::from(440.0));
        voice.note_on(1.into(), 127.into());

        // Skip a few frames in case attack is slow
        let mut buffer = [StereoSample::default(); 5];
        voice.generate(&mut buffer);
        let value = *buffer.last().unwrap();
        assert!(
            value != StereoSample::SILENCE,
            "once triggered, SamplerVoice should make a sound"
        );
    }
}
