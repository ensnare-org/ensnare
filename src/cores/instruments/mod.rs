// Copyright (c) 2024 Mike Tsao

pub use {
    drumkit::DrumkitCore,
    fm::{FmSynthCore, FmSynthCoreBuilder},
    sampler::{SamplerCore, SamplerVoice},
    subtractive::{
        LfoRouting, SubtractiveSynthCore, SubtractiveSynthCoreBuilder, SubtractiveSynthVoice,
        PATCH_DIR as SUBTRACTIVE_PATCH_DIR,
    },
    test::{
        TestAudioSourceCore, TestAudioSourceCoreBuilder, TestControllerAlwaysSendsMidiMessageCore,
    },
};

mod drumkit;
mod fm;
mod sampler;
mod subtractive;
mod test;
