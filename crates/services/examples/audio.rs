// Copyright (c) 2024 Mike Tsao

use clap::Parser;
use derivative::Derivative;
use ensnare::traits::ProvidesService;
use ensnare_services::{prelude::*, AudioStereoSampleType};
use std::{f32::consts::PI, sync::Arc};

#[derive(clap::Parser, Debug, Default)]
#[clap(author, about, long_about = None)]
struct Args {
    /// The frequency to play
    #[clap(short = 'f', long, value_parser)]
    frequency: Option<f32>,
}

/// This struct doesn't use (f32, f32) to represent a stereo sample. We've
/// designed it that way to show how to convert to the expected
/// [AudioStereoSampleType].
struct ToneSample((i16, i16));
impl ToneSample {
    fn left(&self) -> i16 {
        self.0 .0
    }
    fn right(&self) -> i16 {
        self.0 .1
    }
}
#[derive(Derivative)]
#[derivative(Default)]
struct ToneGenerator {
    #[derivative(Default(value = "440.0"))]
    frequency: f32,
    frame_count: f32,
    sample_rate: f32,
    buffer: Vec<ToneSample>,
}
impl ToneGenerator {
    #[allow(dead_code)]
    fn new_with(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            ..Default::default()
        }
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn generate(&mut self, frame_count: usize) {
        self.buffer.clear();
        for _ in 0..frame_count {
            let sample_value = (((self.frame_count / self.sample_rate) * self.frequency * 2.0 * PI)
                .sin()
                * i16::MAX as f32) as i16;
            self.buffer.push(ToneSample((sample_value, sample_value)));
            self.frame_count += 1.0;
        }
    }

    fn buffer(&self) -> &[ToneSample] {
        &self.buffer
    }

    fn frame_count(&self) -> f32 {
        self.frame_count
    }

    fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Instantiate the service. If all is well, then it will send us a
    // [CpalAudioServiceEvent::Reset] event and then begin playing immediately.
    //
    // Note that we need to keep the service in scope even though we don't
    // interact with it after setup. If we don't keep the reference, it will be
    // dropped.
    let service = CpalAudioService::default();
    let sender = service.sender().clone();
    let receiver = service.receiver().clone();

    let mut tone_generator = ToneGenerator::default();
    if let Some(frequency) = args.frequency {
        tone_generator.set_frequency(frequency);
    }

    while let Ok(event) = receiver.recv() {
        match event {
            // The audio service has been reconfigured (or, more likely,
            // started). Handle the new audio stream parameters.
            CpalAudioServiceEvent::Reset(new_sample_rate, new_channel_count) => {
                assert_eq!(
                    new_channel_count, 2,
                    "This example supports only stereo streams"
                );
                println!("Generating audio for {new_channel_count} channels at sample rate {new_sample_rate}Hz");

                // Tell the tone generator that it needs to generate audio at a
                // new sample rate.
                tone_generator.set_sample_rate(new_sample_rate as f32);
            }
            CpalAudioServiceEvent::FramesNeeded(frame_count) => {
                assert_ne!(
                    tone_generator.sample_rate(),
                    0.0,
                    "We shouldn't get a FramesNeeded before Reset"
                );
                assert_ne!(
                    frame_count, 0,
                    "FramesNeeded will never ask for zero frames"
                );

                // Create the buffer of audio.
                tone_generator.generate(frame_count);
                // Send the audio buffer to the service.
                let _ = sender.send(ensnare_services::CpalAudioServiceInput::Frames(Arc::new(
                    tone_generator
                        .buffer()
                        .iter()
                        .map(|s| {
                            let converted_sample: AudioStereoSampleType = (
                                s.left() as f32 / i16::MAX as f32,
                                s.right() as f32 / i16::MAX as f32,
                            );
                            converted_sample
                        })
                        .collect(),
                )));

                // Have we run long enough?
                if tone_generator.frame_count() / tone_generator.sample_rate > 3.0 {
                    let _ = sender.send(ensnare_services::CpalAudioServiceInput::Quit);
                    break;
                }
            }
            CpalAudioServiceEvent::Underrun => eprintln!("FYI buffer underrun"),
        }
    }

    Ok(())
}
