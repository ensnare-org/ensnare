// Copyright (c) 2024 Mike Tsao

use crate::{
    traits::{Configurable, ControlEventsFn, Controls, HandlesMidi, MidiMessagesFn, WorkEvent},
    types::{u7, MidiChannel, MidiMessage, TimeRange},
};
use bit_vec::BitVec;
use core::fmt::Debug;

/// Provides MIDI-related utility functionality.
pub struct MidiUtils {}
impl MidiUtils {
    /// Convenience function to make a note-on [MidiMessage].
    pub fn new_note_on(note: u8, vel: u8) -> MidiMessage {
        MidiMessage::NoteOn {
            key: u7::from(note),
            vel: u7::from(vel),
        }
    }

    /// Convenience function to make a note-off [MidiMessage].
    pub fn new_note_off(note: u8, vel: u8) -> MidiMessage {
        MidiMessage::NoteOff {
            key: u7::from(note),
            vel: u7::from(vel),
        }
    }

    /// If the given message is a note-on velocity-zero MIDI message, translates
    /// it to a plain note-off message. Otherwise passes through the given
    /// message.
    ///
    /// Some MIDI controllers never send a MIDI note-off message (9c nn 00),
    /// instead sending a MIDI note-on message (8c nn vv channel/note/velocity)
    /// with velocity zero. This requires MIDI message handlers to handle two
    /// paths for the same note-off case. This code unites both paths, so
    /// handlers can be simpler.
    pub fn substitute_note_off_for_note_on_vel_zero(message: MidiMessage) -> MidiMessage {
        match message {
            MidiMessage::NoteOn { key, vel } if vel == 0 => MidiMessage::NoteOff {
                key,
                vel: u7::from(0),
            },
            _ => message,
        }
    }
}

/// [MidiNoteMinder] watches a MIDI message stream and remembers which notes are
/// currently active (we've gotten a note-on without a note-off). Then, when
/// asked, it produces a list of MIDI message that turn off all active notes.
///
/// [MidiNoteMinder] doesn't know about [MidiChannel]s. It's up to the caller to
/// track channels, or else assume that if we got any message, it's for us, and
/// that the same is true for recipients of whatever we send.
#[derive(Debug)]
pub struct MidiNoteMinder {
    active_notes: BitVec,
}
impl Default for MidiNoteMinder {
    fn default() -> Self {
        Self {
            active_notes: BitVec::from_elem(128, false),
        }
    }
}
impl HandlesMidi for MidiNoteMinder {
    fn handle_midi_message(
        &mut self,
        _channel: MidiChannel,
        message: MidiMessage,
        _: &mut MidiMessagesFn,
    ) {
        #[allow(unused_variables)]
        match message {
            MidiMessage::NoteOff { key, vel } => {
                self.active_notes.set(key.as_int() as usize, false);
            }
            MidiMessage::NoteOn { key, vel } => {
                self.active_notes
                    .set(key.as_int() as usize, vel != u7::from(0));
            }
            _ => {}
        }
    }
}
impl Controls for MidiNoteMinder {
    fn update_time_range(&mut self, _: &TimeRange) {}

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        for (i, active_note) in self.active_notes.iter().enumerate() {
            if active_note {
                control_events_fn(WorkEvent::Midi(
                    MidiChannel::default(),
                    MidiMessage::NoteOff {
                        key: u7::from_int_lossy(i as u8),
                        vel: u7::from(0),
                    },
                ));
            }
        }
        self.active_notes.clear();
    }
}
impl Configurable for MidiNoteMinder {}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn gather_all_messages(mnm: &mut MidiNoteMinder) -> Vec<MidiMessage> {
        let mut v = Vec::default();
        mnm.work(&mut |e| match e {
            WorkEvent::Midi(_, message) => v.push(message),
            WorkEvent::MidiForTrack(_, _, message) => v.push(message),
            WorkEvent::Control(_) => panic!("didn't expect a Control event here"),
        });
        v
    }
    #[test]
    fn midi_note_minder() {
        let mut mnm = MidiNoteMinder::default();

        assert!(gather_all_messages(&mut mnm).is_empty());

        // Unexpected note-off doesn't explode
        mnm.handle_midi_message(
            MidiChannel::default(),
            MidiUtils::new_note_off(42, 111),
            &mut |_, _| {},
        );
        assert!(gather_all_messages(&mut mnm).is_empty());

        // normal
        mnm.handle_midi_message(
            MidiChannel::default(),
            MidiUtils::new_note_on(42, 99),
            &mut |_, _| {},
        );
        let msgs = gather_all_messages(&mut mnm);
        assert_eq!(msgs.len(), 1);
        assert_eq!(
            msgs[0],
            MidiMessage::NoteOff {
                key: u7::from(42),
                vel: u7::from(0)
            }
        );

        // duplicate on doesn't explode or add twice
        mnm.handle_midi_message(
            MidiChannel::default(),
            MidiUtils::new_note_on(42, 88),
            &mut |_, _| {},
        );
        let msgs = gather_all_messages(&mut mnm);
        assert_eq!(msgs.len(), 1);
        assert_eq!(
            msgs[0],
            MidiMessage::NoteOff {
                key: u7::from(42),
                vel: u7::from(0)
            }
        );

        // normal
        mnm.handle_midi_message(
            MidiChannel::default(),
            MidiUtils::new_note_off(42, 77),
            &mut |_, _| {},
        );
        assert!(gather_all_messages(&mut mnm).is_empty());

        // duplicate off doesn't explode
        mnm.handle_midi_message(
            MidiChannel::default(),
            MidiUtils::new_note_off(42, 66),
            &mut |_, _| {},
        );
        assert!(gather_all_messages(&mut mnm).is_empty());

        // velocity zero treated same as note-off
        mnm.handle_midi_message(
            MidiChannel::default(),
            MidiUtils::new_note_on(42, 99),
            &mut |_, _| {},
        );
        assert_eq!(gather_all_messages(&mut mnm).len(), 1);
        mnm.handle_midi_message(
            MidiChannel::default(),
            MidiUtils::new_note_off(42, 99),
            &mut |_, _| {},
        );
        assert!(gather_all_messages(&mut mnm).is_empty());
        mnm.handle_midi_message(
            MidiChannel::default(),
            MidiUtils::new_note_on(42, 99),
            &mut |_, _| {},
        );
        assert_eq!(gather_all_messages(&mut mnm).len(), 1);
        mnm.handle_midi_message(
            MidiChannel::default(),
            MidiUtils::new_note_on(42, 0),
            &mut |_, _| {},
        );
        assert!(gather_all_messages(&mut mnm).is_empty());
    }

    #[test]
    fn midi_substitution_works() {
        {
            let m = MidiUtils::new_note_on(12, 34);
            assert_eq!(m, MidiUtils::substitute_note_off_for_note_on_vel_zero(m));
        }
        {
            let m = MidiUtils::new_note_on(12, 0);
            assert_eq!(
                MidiUtils::new_note_off(12, 0),
                MidiUtils::substitute_note_off_for_note_on_vel_zero(m)
            );
        }
    }
}
