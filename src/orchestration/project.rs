// Copyright (c) 2024 Mike Tsao

//! Representation of a whole music project, including support for
//! serialization.

use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// A musical piece. Also knows how to render the piece to digital audio.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectV2 {}
#[allow(unused_variables)]
impl Projects for ProjectV2 {
    fn mint_track_uid(&self) -> TrackUid {
        todo!()
    }

    fn create_track(&mut self) -> anyhow::Result<TrackUid> {
        todo!()
    }

    fn delete_track(&mut self, track_uid: TrackUid) -> anyhow::Result<()> {
        todo!()
    }

    fn track_uids(&self) -> &[TrackUid] {
        todo!()
    }

    fn set_track_position(
        &mut self,
        track_uid: TrackUid,
        new_position: usize,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn is_track_muted(&mut self, track_uid: TrackUid) -> bool {
        todo!()
    }

    fn mute_track(&mut self, track_uid: TrackUid, should_mute: bool) {
        todo!()
    }

    fn solo_track(&self) -> Option<TrackUid> {
        todo!()
    }

    fn set_solo_track(&mut self, track_uid: Option<TrackUid>) {
        todo!()
    }

    fn mint_entity_uid(&self) -> Uid {
        todo!()
    }

    fn add_entity(&mut self, track_uid: TrackUid, entity: Box<dyn Entity>) -> anyhow::Result<Uid> {
        todo!()
    }

    fn delete_entity(&mut self, entity_uid: Uid) -> anyhow::Result<()> {
        todo!()
    }

    fn remove_entity(&mut self, entity_uid: Uid) -> anyhow::Result<Box<dyn Entity>> {
        todo!()
    }

    fn entity_uids(&self, track_uid: TrackUid) -> Option<&[Uid]> {
        todo!()
    }

    fn track_for_entity(&self, uid: Uid) -> Option<TrackUid> {
        todo!()
    }

    fn move_entity(
        &mut self,
        uid: Uid,
        new_track_uid: Option<TrackUid>,
        new_position: Option<usize>,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn generate_audio(
        &mut self,
        frames: &mut [StereoSample],
        midi_events_fn: Option<&mut MidiMessagesFn>,
    ) {
        todo!()
    }

    fn generate_and_dispatch_audio(
        &mut self,
        count: usize,
        midi_events_fn: Option<&mut MidiMessagesFn>,
    ) {
        todo!()
    }
}
impl Controls for ProjectV2 {}
impl Configurable for ProjectV2 {
    fn sample_rate(&self) -> SampleRate {
        SampleRate::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::tests::test_trait_configurable;

    #[test]
    fn project_mainline() {
        let p = ProjectV2::default();

        assert_eq!(p.sample_rate(), SampleRate::from(44100))
    }

    #[ignore = "we'll get to this soon"]
    #[test]
    fn project_adheres_to_trait_tests() {
        // test_trait_projects(ProjectV2::default());
        test_trait_configurable(ProjectV2::default());
    }
}
