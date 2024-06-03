// Copyright (c) 2024 Mike Tsao

/// The most commonly used imports.
pub mod prelude {
    #[cfg(feature = "audio")]
    pub use super::{CpalAudioService, CpalAudioServiceEvent, CpalAudioServiceInput};
    pub use super::{CrossbeamChannel, ProvidesService};
}

#[cfg(feature = "audio")]
pub use audio::{
    AudioSampleType, AudioStereoSampleType, CpalAudioService, CpalAudioServiceEvent,
    CpalAudioServiceInput,
};
pub use traits::ProvidesService;
pub use types::CrossbeamChannel;

#[cfg(feature = "audio")]
mod audio;
mod traits;
mod types;
