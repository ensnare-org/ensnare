// Copyright (c) 2024 Mike Tsao

/// The most commonly used imports.
pub mod prelude {
    #[cfg(feature = "audio")]
    pub use super::{CpalAudioService, CpalAudioServiceEvent, CpalAudioServiceInput};
}

#[cfg(feature = "audio")]
pub use audio::{
    AudioSampleType, AudioStereoSampleType, CpalAudioService, CpalAudioServiceEvent,
    CpalAudioServiceInput,
};

#[cfg(feature = "audio")]
mod audio;
