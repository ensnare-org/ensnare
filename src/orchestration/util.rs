// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use std::path::PathBuf;

/// Exports [Projects] to various formats.
pub struct ProjectExporter {}
impl ProjectExporter {
    /// Renders the project as a WAV file to the specified path.
    #[cfg(feature = "hound")]
    pub fn export_to_wav(project: &mut impl Projects, path: PathBuf) -> anyhow::Result<()> {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: project.sample_rate().into(),
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec)?;

        project.skip_to_start();

        let mut renderer = project.render();
        while let Some(frame) = renderer.next() {
            let (left, right) = frame.into_i16();
            let _ = writer.write_sample(left);
            let _ = writer.write_sample(right);
        }

        Ok(())
    }
}
