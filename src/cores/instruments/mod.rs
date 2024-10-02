// Copyright (c) 2024 Mike Tsao

#[cfg(feature = "hound")]
pub use {
    drumkit::DrumkitCore,
    sampler::{SamplerCore, SamplerVoice},
};
pub use {
    fm::{FmSynthCore, FmSynthCoreBuilder},
    subtractive::{
        LfoRouting, SubtractiveSynthCore, SubtractiveSynthCoreBuilder, SubtractiveSynthVoice,
        PATCH_DIR as SUBTRACTIVE_PATCH_DIR,
    },
    test::{
        TestAudioSourceCore, TestAudioSourceCoreBuilder, TestControllerAlwaysSendsMidiMessageCore,
    },
};

#[cfg(feature = "hound")]
mod drumkit;
mod fm;

#[cfg(feature = "hound")]
mod sampler;
mod subtractive;
mod test;
