// Copyright (c) 2024 Mike Tsao

//! The `hello-world` example shows how to use basic crate functionality.

use clap::Parser;
use delegate::delegate;
use derivative::Derivative;
use ensnare::{cores::TimerCore, prelude::*, traits::Displays};
use ensnare_proc_macros::{Control, IsEntity, Metadata};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// The program's command-line arguments.
#[derive(clap::Parser, Debug, Default)]
#[clap(author, about, long_about = None)]
struct Args {
    /// Print version and exit
    #[clap(short = 'v', long, value_parser)]
    version: bool,

    output_filename: Option<String>,
}

/// [MyInstrument] emits a simple tone.
#[derive(Debug, Derivative, IsEntity, Control, Metadata, Serialize, Deserialize)]
#[derivative(Default)]
struct MyInstrument {
    uid: Uid,

    #[derivative(Default(value = "440.0"))]
    frequency: SampleType,

    #[serde(skip)]
    frame_count: usize,
}
impl TransformsAudio for MyInstrument {}
impl Serializable for MyInstrument {}
impl HandlesMidi for MyInstrument {}
impl Generates<StereoSample> for MyInstrument {
    fn generate(&mut self, values: &mut [StereoSample]) -> bool {
        for value in values.iter_mut() {
            let sample = Sample::from(self.frequency * usize_to_sample_type(self.frame_count) / PI);
            self.frame_count += 1;
            *value = sample.into();
        }
        values.fill(<StereoSample>::default());
        true
    }
}
impl Configurable for MyInstrument {}
impl Displays for MyInstrument {}
impl Controls for MyInstrument {}

/// [MyEffect] makes the input audio quieter.
#[derive(Debug, Default, IsEntity, Control, Metadata, Serialize, Deserialize)]
struct MyEffect {
    uid: Uid,
}
impl TransformsAudio for MyEffect {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        input_sample * 0.5
    }
}
impl Serializable for MyEffect {}
impl HandlesMidi for MyEffect {}
impl Generates<StereoSample> for MyEffect {}
impl Configurable for MyEffect {}
impl Displays for MyEffect {}
impl Controls for MyEffect {}

/// [MyController] ends the composition after one measure.
#[derive(Debug, Derivative, IsEntity, Control, Metadata, Serialize, Deserialize)]
#[derivative(Default)]
struct MyController {
    uid: Uid,
    #[derivative(Default(value = "TimerCore::new_with(MusicalTime::FOUR_FOUR_MEASURE)"))]
    timer: TimerCore,
}
impl TransformsAudio for MyController {}
impl Serializable for MyController {}
impl HandlesMidi for MyController {}
impl Generates<StereoSample> for MyController {}
impl Configurable for MyController {}
impl Displays for MyController {}
impl Controls for MyController {
    delegate! {
        to self.timer {
            fn time_range(&self) -> Option<TimeRange>;
            fn update_time_range(&mut self, time_range: &TimeRange);
            fn work(&mut self, control_events_fn: &mut ControlEventsFn);
            fn is_finished(&self) -> bool;
            fn play(&mut self);
            fn stop(&mut self);
            fn skip_to_start(&mut self);
            fn is_performing(&self) -> bool;
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.version {
        eprintln!("{}", ensnare::app_version());
        return Ok(());
    }
    let output_filename = args.output_filename.unwrap_or("output.wav".to_string());

    // The system needs a working buffer for audio.
    let _buffer = [StereoSample::SILENCE; 64];

    // A project contains all the the instruments, controllers, and effects, and
    // their relationships, and uses them to produce a song.
    let mut project = BasicProject::default();

    // It also owns the sample rate and propagates it to the devices that it
    // controls.
    project.update_sample_rate(SampleRate::DEFAULT);

    // It manages a set of Tracks, which are what actually contains musical
    // devices.
    let track_uid = project.create_track().unwrap();

    // TODO: add musical content to be played on the default MIDI channel.

    // Instruments are MIDI-driven entities that emit sounds.
    let synth = MyInstrument::default();
    // Adding an entity to a track forms a chain that sends MIDI, control, and
    // audio data appropriately.
    let _synth_uid = project.add_entity(track_uid, Box::new(synth)).unwrap();

    // Effects process audio.
    let effect = MyEffect::default();
    let _effect_uid = project.add_entity(track_uid, Box::new(effect)).unwrap();

    // Controllers control other entities by emitting MIDI messages and control
    // events. They also determine whether a composition is still playing.
    let controller = MyController::default();
    let _controller_uid = project.add_entity(track_uid, Box::new(controller)).unwrap();

    // Once everything is set up, render an audio stream to disk.
    render_to_disk(&mut project, &output_filename)?;

    Ok(())
}

fn render_to_disk(project: &mut BasicProject, output_filename: &str) -> anyhow::Result<()> {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: project.sample_rate().into(),
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(std::path::PathBuf::from(output_filename), spec)?;
    project.skip_to_start();

    let mut renderer = project.render();
    while let Some(frame) = renderer.next() {
        let (left, right) = frame.into_i16();
        let _ = writer.write_sample(left);
        let _ = writer.write_sample(right);
    }
    Ok(())
}
