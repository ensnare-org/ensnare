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

    // Same with BPM.
    project.update_tempo(Tempo(120.0));

    // Same with time signature.
    project.update_time_signature(TimeSignature::COMMON_TIME);

    // The project also manages a set of Tracks. A Track contains "Entities,"
    // which are the instruments, effects, and controllers that generate and
    // process music and audio data.
    let track_uid = project.create_track().unwrap();

    // Now add several entities to the first track of the project. Adding an
    // entity to a track forms a chain that sends MIDI, control, and audio data
    // appropriately.
    //
    // We'll add one of each kind.
    //
    // A Controller controls other entities by emitting MIDI messages and
    // control events. It also determines whether a composition is still
    // playing. A MIDI sequencer is an example of a Controller.
    //
    // An Instrument is a MIDI-driven entity that emits sounds. A synthesizer
    // is an example of an Instrument.
    //
    // An Effect processes audio. A reverb or delay is an example of an
    // Effect.
    let _controller_id = project.add_entity(track_uid, Box::new(SimpleController::default()));
    let _instrument_id = project.add_entity(track_uid, Box::new(SimpleInstrument::default()));
    let _effect_id = project.add_entity(track_uid, Box::new(SimpleEffect::default()));

    // The SimpleController that we added controls the playback length of this
    // project. It is hard-coded to last one musical measure. During that
    // measure, it emits hardcoded MIDI messages.
    //
    // The SimpleInstrument in tis project emits noise, ignoring MIDI and
    // control inputs. TODO: make it more like a synth that responds to MIDI
    // messages.
    //
    // The project's SimpleEffect multiplies the input by 0.5.

    // At this point, everything is set up for playback. Render the project's
    // audio stream to disk.
    render_to_disk(
        &mut project,
        &args.output_filename.unwrap_or("output.wav".to_string()),
    )?;

    Ok(())
}

// render_to_disk() moves the project's cursor to the start, and asks the
// project to create an Iterator that renders the composition as PCM audio data.
// As expected, the Iterator ends when the composition ends.
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
