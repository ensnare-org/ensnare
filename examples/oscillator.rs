// Copyright (c) 2024 Mike Tsao

//! The `oscillator` example uses an [Oscillator] to produce a monophonic WAV
//! file consisting of a sawtooth wave.
//!
//! The purpose of this example is to show how to use Ensnare's low-level
//! [ensnare::elements] through common traits.

use ensnare::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut oscillator = OscillatorBuilder::default()
        .waveform(Waveform::Sawtooth)
        .frequency(440.0.into())
        .build()
        .unwrap();

    oscillator.update_sample_rate(SampleRate::DEFAULT);

    // At this point, everything is set up for playback. Render the project's
    // audio stream to disk.
    render_to_disk(oscillator, "oscillator-output.wav")?;

    Ok(())
}

// render_to_disk() asks the oscillator for two seconds of samples and writes them to a WAV file.
fn render_to_disk(
    mut generates: impl Generates<BipolarNormal>,
    output_filename: &str,
) -> anyhow::Result<()> {
    let sample_rate: u32 = generates.sample_rate().into();
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(std::path::PathBuf::from(output_filename), spec)?;

    // Continue for two seconds.
    for _ in 0..sample_rate * 2 {
        let mut values = [BipolarNormal::zero(); 1];
        let _ = generates.generate(&mut values);
        let as_sample: Sample = values[0].into();
        let as_i16: i16 = as_sample.into();
        let _ = writer.write_sample(as_i16);
    }
    Ok(())
}
