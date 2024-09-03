// Copyright (c) 2024 Mike Tsao

use super::TimerCore;
use crate::prelude::*;
use delegate::delegate;
use derive_builder::Builder;
use ensnare_proc_macros::Control;
use serde::{Deserialize, Serialize};

// TODO: needs tests!
/// Issues a control event after a specified amount of time.
#[derive(Debug, Default, Builder, Control, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TriggerCore {
    /// The [TimerCore] that causes the trigger.
    timer: TimerCore,

    /// The [ControlValue] to issue.
    pub value: ControlValue,

    #[serde(skip)]
    #[builder(setter(skip))]
    e: TriggerCoreEphemerals,
}
#[derive(Debug, Default)]
pub struct TriggerCoreEphemerals {
    has_triggered: bool,
    is_performing: bool,
}
impl Serializable for TriggerCore {}
impl Controls for TriggerCore {
    fn update_time_range(&mut self, range: &TimeRange) {
        self.timer.update_time_range(range)
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if self.timer.is_finished() && self.e.is_performing && !self.e.has_triggered {
            self.e.has_triggered = true;
            control_events_fn(WorkEvent::Control(self.value));
        }
    }

    fn is_finished(&self) -> bool {
        self.timer.is_finished()
    }

    fn play(&mut self) {
        self.e.is_performing = true;
        self.timer.play();
    }

    fn stop(&mut self) {
        self.e.is_performing = false;
        self.timer.stop();
    }

    fn skip_to_start(&mut self) {
        self.e.has_triggered = false;
        self.timer.skip_to_start();
    }
}
impl Configurable for TriggerCore {
    delegate! {
        to self.timer {
            fn sample_rate(&self) -> SampleRate;
            fn update_sample_rate(&mut self, sample_rate: SampleRate);
            fn tempo(&self) -> Tempo;
            fn update_tempo(&mut self, tempo: Tempo);
            fn time_signature(&self) -> TimeSignature;
            fn update_time_signature(&mut self, time_signature: TimeSignature);
        }
    }
}
impl HandlesMidi for TriggerCore {}
impl TriggerCore {
    fn new_with(timer: TimerCore, value: ControlValue) -> Self {
        Self {
            timer,
            value,
            e: Default::default(),
        }
    }

    /// Returns the [ControlValue] to issue.
    pub fn value(&self) -> ControlValue {
        self.value
    }

    /// Sets the [ControlValue] that will be issued.
    pub fn set_value(&mut self, value: ControlValue) {
        self.value = value;
    }
}
