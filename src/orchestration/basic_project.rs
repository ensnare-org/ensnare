// Copyright (c) 2024 Mike Tsao

//! Representation of a whole music project, including support for
//! serialization.

use crate::{egui::TargetInstrument, prelude::*};
use anyhow::anyhow;
use delegate::delegate;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Indicates what should be shown in the track view.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TrackViewMode {
    /// Shows arranged note [Pattern]s.
    #[default]
    Composition,
    /// Shows one [SignalPath].
    Control(PathUid),
}

/// Utility
#[allow(missing_docs)]
#[derive(Debug)]
pub struct SignalChainItem {
    pub uid: Uid,
    pub name: String,
    pub is_control_source: bool,
}

/// Temporary information associated with each track.
#[allow(missing_docs)]
#[derive(Debug, Default)]
pub struct TrackInfo {
    pub signal_chain: Vec<SignalChainItem>,
    pub targets: Vec<TargetInstrument>,
}

/// A musical piece. Also knows how to render the piece to digital audio.
/// [BasicProject] implements a simple version of [Projects].
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BasicProject {
    track_uid_factory: TrackUidFactory,
    track_uids: Vec<TrackUid>,
    track_mute_state: HashSet<TrackUid>,
    track_solo_state: Option<TrackUid>,

    entity_uid_factory: EntityUidFactory,
    entity_uid_to_entity: HashMap<Uid, Box<dyn Entity>>,
    entity_uid_to_track_uid: HashMap<Uid, TrackUid>,
    track_uid_to_entity_uids: HashMap<TrackUid, Vec<Uid>>,

    transport: Transport,
}
impl Projects for BasicProject {
    fn mint_track_uid(&self) -> TrackUid {
        self.track_uid_factory.mint_next()
    }

    fn create_track(&mut self) -> anyhow::Result<TrackUid> {
        let track_uid = self.mint_track_uid();
        self.track_uids.push(track_uid);
        Ok(track_uid)
    }

    fn delete_track(&mut self, track_uid: TrackUid) -> anyhow::Result<()> {
        if let Some(uids) = self.track_uid_to_entity_uids.get(&track_uid) {
            for uid in uids.clone() {
                let _ = self.delete_entity(uid);
            }
        }
        let _ = self.track_uid_to_entity_uids.remove(&track_uid);
        self.track_uids.retain(|tuid| &track_uid != tuid);
        Ok(())
    }

    fn track_uids(&self) -> &[TrackUid] {
        &self.track_uids
    }

    fn set_track_position(
        &mut self,
        track_uid: TrackUid,
        new_position: usize,
    ) -> anyhow::Result<()> {
        if self.track_uids.contains(&track_uid) {
            if new_position <= self.track_uids.len() {
                self.delete_track(track_uid)?;
                self.track_uids.insert(new_position, track_uid);
                Ok(())
            } else {
                Err(anyhow!(
                    "Invalid track position {new_position} for {track_uid}"
                ))
            }
        } else {
            Err(anyhow!("Track {track_uid} not found"))
        }
    }

    fn is_track_muted(&mut self, track_uid: TrackUid) -> bool {
        self.track_mute_state.contains(&track_uid)
    }

    fn mute_track(&mut self, track_uid: TrackUid, should_mute: bool) {
        if should_mute {
            self.track_mute_state.insert(track_uid);
        } else {
            self.track_mute_state.remove(&track_uid);
        }
    }

    fn solo_track(&self) -> Option<TrackUid> {
        self.track_solo_state
    }

    fn set_solo_track(&mut self, track_uid: Option<TrackUid>) {
        self.track_solo_state = track_uid
    }

    fn mint_entity_uid(&self) -> Uid {
        self.entity_uid_factory.mint_next()
    }

    fn add_entity(
        &mut self,
        track_uid: TrackUid,
        mut entity: Box<dyn Entity>,
    ) -> anyhow::Result<Uid> {
        if !self.track_uids.contains(&track_uid) {
            return Err(anyhow!("Nonexistent track {track_uid}"));
        }
        let uid = if entity.uid() != Uid::default() {
            entity.uid()
        } else {
            let uid = self.mint_entity_uid();
            entity.set_uid(uid);
            uid
        };
        self.entity_uid_to_entity.insert(uid.clone(), entity);
        self.entity_uid_to_track_uid.insert(uid.clone(), track_uid);
        self.track_uid_to_entity_uids
            .entry(track_uid)
            .or_default()
            .push(uid.clone());
        Ok(uid)
    }

    fn delete_entity(&mut self, uid: Uid) -> anyhow::Result<()> {
        let _ = self.remove_entity(uid);
        Ok(())
    }

    fn remove_entity(&mut self, uid: Uid) -> anyhow::Result<Box<dyn Entity>> {
        if let Some(track_uid) = self.entity_uid_to_track_uid.remove(&uid) {
            if let Some(entities) = self.track_uid_to_entity_uids.get_mut(&track_uid) {
                entities.retain(|e| *e != uid);
            }
        }
        if let Some(entity) = self.entity_uid_to_entity.remove(&uid) {
            Ok(entity)
        } else {
            Err(anyhow!("No such entity {uid}"))
        }
    }

    fn entity_uids(&self, track_uid: TrackUid) -> Option<&[Uid]> {
        if let Some(uids) = self.track_uid_to_entity_uids.get(&track_uid) {
            let uids: &[Uid] = uids;
            Some(uids)
        } else {
            None
        }
    }

    fn track_for_entity(&self, uid: Uid) -> Option<TrackUid> {
        self.entity_uid_to_track_uid.get(&uid).copied()
    }

    fn move_entity(
        &mut self,
        uid: Uid,
        new_track_uid: Option<TrackUid>,
        new_position: Option<usize>,
    ) -> anyhow::Result<()> {
        if !self.entity_uid_to_track_uid.contains_key(&uid) {
            return Err(anyhow!("Entity {uid} not found"));
        }
        if let Some(new_track_uid) = new_track_uid {
            if let Some(old_track_uid) = self.entity_uid_to_track_uid.get(&uid) {
                if *old_track_uid != new_track_uid {
                    if let Some(uids) = self.track_uid_to_entity_uids.get_mut(old_track_uid) {
                        uids.retain(|u| *u != uid);
                        self.track_uid_to_entity_uids
                            .entry(new_track_uid)
                            .or_default()
                            .push(uid);
                    }
                }
            }
            self.entity_uid_to_track_uid.insert(uid, new_track_uid);
        }
        if let Some(new_position) = new_position {
            if let Some(track_uid) = self.entity_uid_to_track_uid.get(&uid) {
                let uids = self.track_uid_to_entity_uids.entry(*track_uid).or_default();
                if new_position <= uids.len() {
                    uids.retain(|u| *u != uid);
                    uids.insert(new_position, uid);
                } else {
                    return Err(anyhow!("new position {new_position} is out of bounds"));
                }
            }
        }
        Ok(())
    }

    fn generate_audio(
        &mut self,
        frames: &mut [StereoSample],
        midi_events_fn: Option<&mut MidiMessagesFn>,
    ) {
        self.handle_controllers(frames.len(), midi_events_fn);
        self.handle_instruments(frames);
        self.handle_effects(frames);
    }

    fn generate_and_dispatch_audio(
        &mut self,
        count: usize,
        mut midi_events_fn: Option<&mut MidiMessagesFn>,
    ) {
        if count == 0 {
            return;
        }
        const BUFFER_LEN: usize = 64;
        let mut buffer = [StereoSample::SILENCE; BUFFER_LEN];
        let mut remaining = count;

        while remaining != 0 {
            let to_generate = remaining.min(BUFFER_LEN);
            let buffer_slice = &mut buffer[0..to_generate];
            buffer_slice.fill(StereoSample::SILENCE);
            self.generate_audio(buffer_slice, midi_events_fn.as_deref_mut());
            remaining -= to_generate;
        }
    }
}
impl Configurable for BasicProject {
    delegate! {
        to self.transport {
            fn sample_rate(&self) -> SampleRate;
            fn tempo(&self) -> Tempo;
            fn time_signature(&self) -> TimeSignature;
        }
    }

    fn update_sample_rate(&mut self, sample_rate: SampleRate) {
        self.transport.update_sample_rate(sample_rate);
        self.entity_uid_to_entity
            .values_mut()
            .for_each(|e| e.update_sample_rate(sample_rate));
    }

    fn update_tempo(&mut self, tempo: Tempo) {
        self.transport.update_tempo(tempo);
        self.entity_uid_to_entity
            .values_mut()
            .for_each(|e| e.update_tempo(tempo));
    }

    fn update_time_signature(&mut self, time_signature: TimeSignature) {
        self.transport.update_time_signature(time_signature);
        self.entity_uid_to_entity
            .values_mut()
            .for_each(|e| e.update_time_signature(time_signature));
    }

    fn reset(&mut self) {
        self.transport.reset();
        self.entity_uid_to_entity
            .values_mut()
            .for_each(|e| e.reset());
    }
}
impl Controls for BasicProject {
    fn update_time_range(&mut self, time_range: &TimeRange) {
        self.transport.update_time_range(time_range);
        for entity in self.entity_uid_to_entity.values_mut() {
            entity.update_time_range(time_range);
        }
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        self.transport.work(control_events_fn);
        self.entity_uid_to_entity
            .values_mut()
            .for_each(|e| e.work(control_events_fn));
    }

    fn is_finished(&self) -> bool {
        self.transport.is_finished() && self.entity_uid_to_entity.values().all(|e| e.is_finished())
    }

    fn play(&mut self) {
        self.transport.play();
        self.entity_uid_to_entity
            .values_mut()
            .for_each(|e| e.play());
    }

    fn stop(&mut self) {
        self.transport.stop();
        self.entity_uid_to_entity
            .values_mut()
            .for_each(|e| e.stop());
    }

    fn skip_to_start(&mut self) {
        self.transport.skip_to_start();
        self.entity_uid_to_entity
            .values_mut()
            .for_each(|e| e.skip_to_start());
    }
}
impl BasicProject {
    fn handle_controllers(
        &mut self,
        frames_len: usize,
        mut midi_events_fn: Option<&mut MidiMessagesFn>,
    ) {
        let is_finished_at_start = self.is_finished();
        let time_range = self.transport.advance(frames_len);
        self.update_time_range(&time_range);
        self.work(&mut |e| match e {
            WorkEvent::Midi(channel, message) => {
                if let Some(midi_events_fn) = midi_events_fn.as_mut() {
                    midi_events_fn(channel, message);
                }
            }
            WorkEvent::MidiForTrack(_, _, _) => todo!(),
            WorkEvent::Control(_) => todo!(),
        });
        let is_finished_at_end = self.is_finished();
        if !is_finished_at_start && is_finished_at_end {
            self.stop();
        }
    }

    fn handle_instruments(&mut self, frames: &mut [StereoSample]) {
        let frame_count = frames.len();

        self.entity_uid_to_entity.values_mut().for_each(|e| {
            let mut track_buffer = Vec::default();
            track_buffer.resize(frame_count, StereoSample::SILENCE);
            e.generate(&mut track_buffer);

            track_buffer
                .iter()
                .zip(frames.iter_mut())
                .for_each(|(src, dst)| {
                    *dst += *src;
                });
        });
    }

    fn handle_effects(&mut self, frames: &mut [StereoSample]) {
        self.entity_uid_to_entity.values_mut().for_each(|e| {
            e.transform(frames);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        orchestration::traits::tests::test_trait_projects, traits::tests::test_trait_configurable,
    };

    #[test]
    fn project_mainline() {
        let p = BasicProject::default();

        assert_eq!(
            p.sample_rate(),
            SampleRate::from(SampleRate::DEFAULT_SAMPLE_RATE)
        );
        assert_eq!(p.tempo(), Tempo::from(Tempo::DEFAULT_TEMPO));
        assert_eq!(
            p.time_signature(),
            TimeSignature::new_with(TimeSignature::DEFAULT_TOP, TimeSignature::DEFAULT_BOTTOM)
                .unwrap()
        );
    }

    #[test]
    fn project_adheres_to_trait_tests() {
        test_trait_projects(BasicProject::default());
        test_trait_configurable(BasicProject::default());
    }
}
