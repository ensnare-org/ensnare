// Copyright (c) 2024 Mike Tsao

//! This example produces a song that is output as a WAV file to the current
//! directory.

use clap::Parser;
use ensnare::{
    entities::{Drumkit, SubtractiveSynth},
    prelude::*,
};
use std::path::PathBuf;

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

    Paths::set_instance(Paths::default());
    init_sample_libraries();
    let factory = BuiltInEntities::register(EntityFactory::default()).finalize();

    let mut project = Project::default();

    project.update_time_signature(TimeSignature::WALTZ_TIME);

    let lead_track_uid = project.create_track().unwrap();
    project.set_track_midi_channel(lead_track_uid, MidiChannel(0));
    let _lead_synth_id = project
        .add_entity(
            lead_track_uid,
            factory
                .new_entity(
                    &EntityKey::from(SubtractiveSynth::ENTITY_KEY),
                    Uid::default(),
                )
                .unwrap(),
        )
        .unwrap();

    let rhythm_track_uid = project.create_track().unwrap();
    project.set_track_midi_channel(rhythm_track_uid, MidiChannel::DRUM);
    let _drumkit_id = project.add_entity(
        rhythm_track_uid,
        factory
            .new_entity(&EntityKey::from(Drumkit::ENTITY_KEY), Uid::default())
            .unwrap(),
    );

    const RR: u8 = PatternBuilder::REST;
    let lead_pattern_uid = project
        .add_pattern(
            PatternBuilder::default()
                .note_sequence(
                    vec![
                        60, RR, RR, 60, RR, 62, RR, RR, RR, 60, RR, RR, //
                        RR, 65, RR, RR, RR, 64, RR, RR, RR, RR, RR, RR, //
                    ],
                    None,
                )
                .build()
                .unwrap(),
            None,
        )
        .unwrap();
    let rhythm_pattern_uid = project
        .add_pattern(
            PatternBuilder::default()
                .note_sequence(
                    vec![
                        RR, RR, RR, RR, RR, 35, RR, RR, RR, RR, RR, RR, //
                        RR, RR, RR, RR, RR, 35, RR, RR, RR, RR, RR, RR, //
                    ],
                    None,
                )
                .note_sequence(
                    vec![
                        RR, RR, RR, RR, RR, RR, RR, RR, RR, RR, RR, RR, //
                        RR, 38, RR, RR, RR, RR, RR, RR, RR, RR, RR, RR, //
                    ],
                    None,
                )
                .build()
                .unwrap(),
            None,
        )
        .unwrap();

    let _lead_arrangement_uid = project
        .arrange_pattern(
            lead_track_uid,
            lead_pattern_uid,
            Some(MidiChannel(0)),
            MusicalTime::START,
        )
        .unwrap();
    let _rhythm_arrangement_uid = project
        .arrange_pattern(
            rhythm_track_uid,
            rhythm_pattern_uid,
            Some(MidiChannel::DRUM),
            MusicalTime::START,
        )
        .unwrap();

    let output_prefix = PathBuf::from("happy-birthday");
    project.save_and_export(output_prefix)
}
