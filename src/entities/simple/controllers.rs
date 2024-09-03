// Copyright (c) 2024 Mike Tsao

use crate::{prelude::*, util::MidiUtils};
use delegate::delegate;
use ensnare_proc_macros::{IsEntity, Metadata};
use serde::{Deserialize, Serialize};

/// A controller that emits a MIDI note and then ends.
#[derive(Debug, Default, IsEntity, Metadata, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[entity(
    Controllable,
    Displays,
    GeneratesStereoSample,
    HandlesMidi,
    Serializable,
    SkipInner,
    TransformsAudio
)]
pub struct SimpleController {
    uid: Uid,

    #[serde(skip)]
    c: Configurables,

    #[serde(skip)]
    time_range: TimeRange,

    #[serde(skip)]
    is_finished: bool,

    #[serde(skip)]
    is_performing: bool,
}
impl SimpleController {
    #[allow(missing_docs)]
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            c: Default::default(),
            time_range: Default::default(),
            is_finished: Default::default(),
            is_performing: Default::default(),
        }
    }
}
impl Configurable for SimpleController {
    delegate! {
        to self.c {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
            fn reset(&mut self);
        }
    }
}
impl Controls for SimpleController {
    fn time_range(&self) -> Option<TimeRange> {
        Some(self.time_range.clone())
    }

    fn update_time_range(&mut self, time_range: &TimeRange) {
        self.time_range = time_range.clone();
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if self.is_performing {
            if self.time_range.contains(&MusicalTime::START) {
                control_events_fn(WorkEvent::Midi(
                    MidiChannel::default(),
                    MidiUtils::new_note_on(60, 127),
                ));
            } else if self.time_range.contains(&MusicalTime::ONE_BEAT) {
                control_events_fn(WorkEvent::Midi(
                    MidiChannel::default(),
                    MidiUtils::new_note_off(60, 127),
                ));
            } else if self.time_range.contains(&MusicalTime::FOUR_FOUR_MEASURE) {
                self.is_finished = true;
            }
        }
    }

    fn is_finished(&self) -> bool {
        self.is_finished
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }

    fn skip_to_start(&mut self) {
        self.time_range = TimeRange::default();
        self.is_finished = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_controller() {
        let c = SimpleController::default();
        assert!(!c.is_finished());
    }
}
