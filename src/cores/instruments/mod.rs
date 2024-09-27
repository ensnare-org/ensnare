// Copyright (c) 2024 Mike Tsao

pub use fm::{FmSynthCore, FmSynthCoreBuilder};
pub use subtractive::{
    LfoRouting, SubtractiveSynthCore, SubtractiveSynthCoreBuilder, SubtractiveSynthVoice,
    PATCH_DIR as SUBTRACTIVE_PATCH_DIR,
};
pub use test::{
    TestAudioSourceCore, TestAudioSourceCoreBuilder, TestControllerAlwaysSendsMidiMessageCore,
};

mod fm;
mod subtractive;
mod test;
