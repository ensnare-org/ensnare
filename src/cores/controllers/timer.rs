// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use delegate::delegate;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

/// Runs for a specified amount of [MusicalTime], then sets [Controls::is_finished()].
#[derive(Debug, Builder, Default, Clone, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TimerCore {
    /// The length of time before this timer expires.
    duration: MusicalTime,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: TimerCoreEphemerals,
}
#[derive(Debug, Default, Clone)]
pub struct TimerCoreEphemerals {
    is_performing: bool,
    is_finished: bool,
    end_time: Option<MusicalTime>,
    c: Configurables,
}
impl Serializable for TimerCore {}
#[allow(missing_docs)]
impl TimerCore {
    pub fn new_with(duration: MusicalTime) -> Self {
        let mut r = TimerCore::default();
        r.set_duration(duration);
        r
    }

    pub fn duration(&self) -> MusicalTime {
        self.duration
    }

    pub fn set_duration(&mut self, duration: MusicalTime) {
        self.duration = duration;
        self.e.is_finished = duration.is_empty();
    }

    fn set_finished_from_time_range(&mut self, range: &TimeRange) {
        if let Some(end_time) = self.e.end_time {
            if range.0.contains(&end_time) {
                self.e.is_finished = true;
            }
        }
    }
}
impl HandlesMidi for TimerCore {}
impl Configurable for TimerCore {
    delegate! {
        to self.e.c {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
    fn reset(&mut self) {
        self.e.is_finished = false;
        // We use the setter to pick up the is_empty() logic.
        self.set_duration(self.duration);
    }
}
impl Controls for TimerCore {
    fn update_time_range(&mut self, range: &TimeRange) {
        if self.e.is_performing {
            if self.e.end_time.is_none() {
                // The first time we're called with an update_time_range() while
                // performing, we take that as the start of the timer. So we set
                // the end time, and then test to see whether we've reached it.
                self.e.end_time = Some(range.0.start + self.duration);
            }
            self.set_finished_from_time_range(range);
        }
    }

    fn is_finished(&self) -> bool {
        self.e.is_finished
    }

    fn play(&mut self) {
        self.e.is_performing = true;
    }

    fn stop(&mut self) {
        self.e.is_performing = false;
    }

    fn is_performing(&self) -> bool {
        self.e.is_performing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_works() {
        {
            let t = TimerCore::new_with(MusicalTime::START);
            assert!(
                t.is_finished(),
                "a zero-length Timer should end immediately, even if not performing"
            );
        }
        {
            let mut t = TimerCore::new_with(MusicalTime::ONE_UNIT);
            assert!(
                !t.is_finished(),
                "a non-zero-length Timer should not end immediately"
            );

            t.update_time_range(&TimeRange::new_with_start_and_duration(
                MusicalTime::START,
                MusicalTime::ONE_UNIT * 2,
            ));
            assert!(
                !t.is_finished(),
                "a Timer shouldn't end if it's not performing."
            );

            t.play();
            t.update_time_range(&TimeRange::new_with_start_and_duration(
                MusicalTime::START,
                MusicalTime::ONE_UNIT * 2,
            ));
            assert!(
                t.is_finished(),
                "a Timer should end if it's performing and we've gone past its expiration."
            );
        }

        {
            let mut t = TimerCore::new_with(MusicalTime::ONE_UNIT);
            t.play();
            t.update_time_range(&TimeRange::new_with_start_and_duration(
                MusicalTime::START,
                MusicalTime::ONE_UNIT,
            ));
            assert!(
                !t.is_finished(),
                "a performing Timer should not end during a work slice ending before the Timer's duration."
            );

            t.update_time_range(&TimeRange::new_with_start_and_duration(
                MusicalTime::ONE_UNIT,
                MusicalTime::ONE_UNIT,
            ));
            assert!(
                t.is_finished(),
                "a performing Timer should end during a work slice including the Timer's duration."
            );
        }

        {
            let mut t = TimerCore::new_with(MusicalTime::ONE_UNIT);
            t.play();
            let later_time_slice = TimeRange::new_with_start_and_duration(
                MusicalTime::ONE_BEAT * 2,
                MusicalTime::ONE_UNIT * 2,
            );
            let earlier_time_slice = TimeRange::new_with_start_and_duration(
                MusicalTime::ONE_BEAT,
                MusicalTime::ONE_UNIT * 2,
            );
            t.update_time_range(&later_time_slice);
            assert!(t.is_finished());
            t.update_time_range(&earlier_time_slice);
            assert!(
                t.is_finished(),
                "Once a Timer has fired, it should remain fired even for earlier time slices."
            );
            t.reset();
            t.play();
            assert!(
                !t.is_finished(),
                "A Timer should return to the not-finished state after a reset()."
            );
        }
    }
}
