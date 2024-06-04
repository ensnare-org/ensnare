// Copyright (c) 2024 Mike Tsao

//! Representation of a whole music project, including support for
//! serialization.

use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// A musical piece. Also knows how to render the piece to digital audio.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectV2 {}
impl Configurable for ProjectV2 {
    fn sample_rate(&self) -> SampleRate {
        SampleRate::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_mainline() {
        let p = ProjectV2::default();

        assert_eq!(p.sample_rate(), SampleRate::from(44100))
    }
}
