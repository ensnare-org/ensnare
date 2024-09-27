// Copyright (c) 2024 Mike Tsao

//! Effects transform audio through the
//! [TransformsAudio](crate::traits::TransformsAudio) trait. Examples are
//! [Reverb] and filters.

pub use bitcrusher::{BitcrusherCore, BitcrusherCoreBuilder};
pub use delay::{DelayCore, DelayCoreBuilder, DelayLine, Delays};
pub use filter::{
    BiQuadFilterAllPassCore, BiQuadFilterAllPassCoreBuilder, BiQuadFilterBandPassCore,
    BiQuadFilterBandPassCoreBuilder, BiQuadFilterBandStopCore, BiQuadFilterBandStopCoreBuilder,
    BiQuadFilterHighPassCore, BiQuadFilterHighPassCoreBuilder, BiQuadFilterHighShelfCoreBuilder,
    BiQuadFilterLowPass24dbCore, BiQuadFilterLowPass24dbCoreBuilder, BiQuadFilterLowShelfCore,
    BiQuadFilterLowShelfCoreBuilder, BiQuadFilterNoneCoreBuilder, BiQuadFilterPeakingEqCore,
    BiQuadFilterPeakingEqCoreBuilder,
};
pub use gain::{GainCore, GainCoreBuilder};
pub use limiter::{LimiterCore, LimiterCoreBuilder};
pub use reverb::{ReverbCore, ReverbCoreBuilder};

mod bitcrusher;
mod delay;
mod filter;
mod gain;
mod limiter;
mod reverb;
