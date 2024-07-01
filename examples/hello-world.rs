// Copyright (c) 2024 Mike Tsao

//! The `hello-world` example shows how to use basic crate functionality.

use clap::Parser;
use ensnare::prelude::*;

/// The program's command-line arguments.
#[derive(clap::Parser, Debug, Default)]
#[clap(author, about, long_about = None)]
struct Args {
    /// Print version and exit
    #[clap(short = 'v', long, value_parser)]
    version: bool,

    output_filename: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.version {
        eprintln!("{}", ensnare::app_version());
        return Ok(());
    }

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

    // Add several entities to the project. Adding an entity to a track forms a
    // chain that sends MIDI, control, and audio data appropriately.

    // Instruments are MIDI-driven entities that emit sounds.
    let _ = project.add_entity(track_uid, Box::new(SimpleInstrument::default()));

    // Effects process audio.
    let _ = project.add_entity(track_uid, Box::new(SimpleEffect::default()));

    // Controllers control other entities by emitting MIDI messages and control
    // events. They also determine whether a composition is still playing.
    let _ = project.add_entity(track_uid, Box::new(SimpleController::default()));

    // Once everything is set up, render an audio stream to disk.
    render_to_disk(
        &mut project,
        &args.output_filename.unwrap_or("output.wav".to_string()),
    )?;

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
