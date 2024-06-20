// Copyright (c) 2024 Mike Tsao

//! The `hello-world` example shows how to use basic crate functionality.

use clap::Parser;
use ensnare::{prelude::*, traits::Displays};
use ensnare_proc_macros::{Control, IsEntity, Metadata};
use serde::{Deserialize, Serialize};

/// The program's command-line arguments.
#[derive(clap::Parser, Debug, Default)]
#[clap(author, about, long_about = None)]
struct Args {
    /// Print version and exit
    #[clap(short = 'v', long, value_parser)]
    version: bool,

    output_filename: Option<String>,
}

#[derive(Debug, Default, IsEntity, Control, Metadata, Serialize, Deserialize)]
struct Instrument {
    uid: Uid,
}
impl TransformsAudio for Instrument {}
impl Serializable for Instrument {}
impl HandlesMidi for Instrument {}
impl Generates<StereoSample> for Instrument {}
impl Configurable for Instrument {}
impl Displays for Instrument {}
impl Controls for Instrument {}

#[derive(Debug, Default, IsEntity, Control, Metadata, Serialize, Deserialize)]
struct Effect {
    uid: Uid,
}
impl TransformsAudio for Effect {}
impl Serializable for Effect {}
impl HandlesMidi for Effect {}
impl Generates<StereoSample> for Effect {}
impl Configurable for Effect {}
impl Displays for Effect {}
impl Controls for Effect {}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.version {
        eprintln!("{}", ensnare::app_version());
        return Ok(());
    }
    let output_filename = args.output_filename.unwrap_or("output.wav".to_string());

    // The system needs a working buffer for audio.
    let _buffer = [StereoSample::SILENCE; 64];

    // Project contains all the the instruments, controllers, and effects, and
    // their relationships, and uses them to produce a song.
    let mut project = BasicProject::default();

    // It also owns the sample rate and propagates it to the devices that it
    // controls.
    project.update_sample_rate(SampleRate::DEFAULT);

    // It manages a set of Tracks, which are what actually contains musical
    // devices.
    let track_uid = project.create_track().unwrap();

    // TODO: add musical content to be played on the default MIDI channel.

    // Instrument is a MIDI instrument that makes simple sounds. Adding an
    // entity to a track forms a chain that sends MIDI, control, and audio data
    // appropriately.
    let synth = Instrument::default();
    let _synth_uid = project.add_entity(track_uid, Box::new(synth)).unwrap();

    // An effect takes the edge off the synth.
    let effect = Effect::default();
    let _effect_uid = project.add_entity(track_uid, Box::new(effect)).unwrap();

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
