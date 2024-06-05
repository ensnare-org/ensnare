// Copyright (c) 2024 Mike Tsao

//! Provides MIDI interface services.

use core::fmt::Debug;
use ensnare::{prelude::*, types::MidiPortDescriptor};

/// The client sends requests to the MIDI interface through [MidiServiceInput]
/// messages.
///
/// "input" and "output" are from the perspective of the MIDI interface. For
/// example, suppose MIDI keyboard K is connected to MIDI interface I, which is
/// connected to PC P, and MIDI synthesizer S is connected to I's output. When
/// the user presses a key on K, it goes *in* to I's *input* and then to P. When
/// P generates a MIDI message, it sends it *out* via I through I's *output* to
/// S.
#[derive(Clone, Debug)]
pub enum MidiServiceInput {
    /// Requests a rescan of the MIDI input/output ports.
    RefreshPorts,

    /// The user has picked a MIDI input. Switch to it.
    ///
    /// A MIDI input leaves the MIDI device and enters the MIDI interface's
    /// input port.
    SelectMidiInput(MidiPortDescriptor),

    /// The user has picked a MIDI output. Switch to it.
    ///
    /// A MIDI output leaves the MIDI interface's output port and enters the
    /// MIDI device.
    SelectMidiOutput(MidiPortDescriptor),

    /// The application wants to send a MIDI message to external hardware.
    Midi(MidiChannel, MidiMessage),

    /// The app is ready to quit, so the service should end.
    Quit,

    /// Attempt to set the selected MIDI input by matching a text description.
    RestoreMidiInput(String),

    /// Attempt to set the selected MIDI output by matching a text description.
    RestoreMidiOutput(String),
}

/// The service provides updates to the client through [MidiServiceEvent]
/// messages.
#[derive(Clone, Debug)]
pub enum MidiServiceEvent {
    /// The MIDI input ports have been updated.
    InputPorts(Vec<MidiPortDescriptor>),

    /// A new input port has been selected.
    InputPortSelected(Option<MidiPortDescriptor>),

    /// The MIDI output ports have been updated.
    OutputPorts(Vec<MidiPortDescriptor>),

    /// A new output port has been selected.
    OutputPortSelected(Option<MidiPortDescriptor>),

    /// A MIDI message has arrived from external hardware.
    Midi(MidiChannel, MidiMessage),

    /// A MIDI message was just dispatched to external hardware. This message
    /// exists to let the UI flash an activity indicator; that's why it doesn't
    /// contain the actual message.
    ///
    /// TODO: is this necessary? The message came from the PC, so why do we need
    /// the interface to tell us what we already know?
    MidiOut,

    /// The MIDI engine has successfully processed [MidiServiceInput::Quit], and
    /// the service will go away shortly.
    Quit,
}

/// Wraps the [midir](https://crates.io/crates/midir) crate with a
/// crossbeam-channels interface.
#[derive(Debug)]
pub struct MidiService {
    inputs: CrossbeamChannel<MidiServiceInput>,
    events: CrossbeamChannel<MidiServiceEvent>,
}
impl Default for MidiService {
    fn default() -> Self {
        Self::new()
    }
}
impl MidiService {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        let r = Self {
            inputs: Default::default(),
            events: Default::default(),
        };

        r.start_thread();
        r
    }

    fn start_thread(&self) {}
}
impl ProvidesService<MidiServiceInput, MidiServiceEvent> for MidiService {
    fn sender(&self) -> &crossbeam::channel::Sender<MidiServiceInput> {
        &self.inputs.sender
    }

    fn receiver(&self) -> &crossbeam::channel::Receiver<MidiServiceEvent> {
        &self.events.receiver
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midi_service() {
        let _s = MidiService::default();
    }
}
