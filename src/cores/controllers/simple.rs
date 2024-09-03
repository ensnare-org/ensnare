// Copyright (c) 2024 Mike Tsao

use crate::prelude::*;
use ensnare_proc_macros::Control;

/// A controller that emits MIDI note-on messages every time [Controls::work()]
/// is called.
#[derive(Debug, Default, Control)]
pub struct SimpleControllerAlwaysSendsMidiMessageCore {
    midi_note: u8,
    is_performing: bool,
}
impl HandlesMidi for SimpleControllerAlwaysSendsMidiMessageCore {}
impl Controls for SimpleControllerAlwaysSendsMidiMessageCore {
    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if self.is_performing {
            control_events_fn(WorkEvent::Midi(
                MidiChannel::default(),
                MidiMessage::NoteOn {
                    key: u7::from(self.midi_note),
                    vel: u7::from(127),
                },
            ));
            self.midi_note += 1;
            if self.midi_note > 127 {
                self.midi_note = 1;
            }
        }
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn play(&mut self) {
        self.is_performing = true;
    }

    fn stop(&mut self) {
        self.is_performing = false;
    }
}
impl Configurable for SimpleControllerAlwaysSendsMidiMessageCore {}
impl Serializable for SimpleControllerAlwaysSendsMidiMessageCore {}
