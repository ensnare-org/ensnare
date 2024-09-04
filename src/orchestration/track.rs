// Copyright (c) 2024 Mike Tsao

use crate::{prelude::*, types::IsUid};
use delegate::delegate;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use synonym::Synonym;

/// Newtype for track title string.
#[derive(Synonym, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[synonym(skip(Default))]
#[serde(rename_all = "kebab-case")]
pub struct TrackTitle(#[derivative(Default(value = "\"Untitled\".to_string()"))] pub String);

/// Identifies a track.
#[derive(Synonym, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[synonym(skip(Default))]
#[serde(rename_all = "kebab-case")]
pub struct TrackUid(#[derivative(Default(value = "1"))] pub usize);
impl IsUid for TrackUid {
    fn as_usize(&self) -> usize {
        self.0
    }
}

/// Mints unique [TrackUid]s.
#[derive(Debug, Serialize, Deserialize)]
pub struct TrackUidFactory(UidFactory<TrackUid>);
impl Default for TrackUidFactory {
    fn default() -> Self {
        Self(UidFactory::<TrackUid>::new(Self::DEFAULT_FIRST_UID_VALUE))
    }
}
impl TrackUidFactory {
    /// The value of the first Uid that a fresh TrackUidFactory should mint.
    pub const DEFAULT_FIRST_UID_VALUE: usize = 1;

    /// The first Uid that a fresh TrackUidFactory should mint.
    pub const DEFAULT_FIRST_UID: TrackUid = TrackUid(Self::DEFAULT_FIRST_UID_VALUE);

    delegate! {
        to self.0 {
            #[allow(missing_docs)]
            pub fn mint_next(&self) -> TrackUid;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mainline() {
        let factory = TrackUidFactory::default();

        let track_uid = factory.mint_next();
        assert_eq!(track_uid, TrackUidFactory::DEFAULT_FIRST_UID);
        let track_uid_2 = factory.mint_next();
        assert_ne!(track_uid, track_uid_2);
    }
}
