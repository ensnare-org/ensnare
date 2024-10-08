// Copyright (c) 2024 Mike Tsao

//! Provides MIDI-interface services.

use core::fmt::Debug;
use crossbeam::channel::Select;
use ensnare::{prelude::*, types::MidiPortDescriptor, util::MidiUtils};
use midir::{
    MidiInput, MidiInputConnection, MidiInputPort, MidiOutput, MidiOutputConnection, MidiOutputPort,
};
use midly::live::LiveEvent;

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
    /// Scans the MIDI input/output ports. The scan results will be provided in
    /// [MidiServiceEvent::InputPorts] and [MidiServiceEvent::OutputPorts].
    RefreshPorts,

    /// Switch to the given MIDI input port, or None to disconnect from any
    /// currently connected input port.
    SelectInputPort(Option<MidiPortDescriptor>),

    /// Switch to the given MIDI output port, or None to disconnect from any
    /// currently connected output port.
    SelectOutputPort(Option<MidiPortDescriptor>),

    /// The application wants to send a MIDI message to external hardware.
    Midi(MidiChannel, MidiMessage),

    /// The app is ready to quit, so the service should end.
    Quit,
}

/// The service provides updates to the client through [MidiServiceEvent]
/// messages.
#[derive(Clone, Debug)]
pub enum MidiServiceEvent {
    /// Provides the results of the most recent scan of available MIDI input
    /// ports. Normally sent in response to [MidiServiceInput::RefreshPorts].
    InputPorts(Vec<MidiPortDescriptor>),

    /// Provides the results of the most recent scan of available MIDI output
    /// ports. Normally sent in response to [MidiServiceInput::RefreshPorts].
    OutputPorts(Vec<MidiPortDescriptor>),

    /// A new input port has been selected, or None if the active port has been
    /// disconnected.
    InputPortSelected(Option<MidiPortDescriptor>),

    /// A new output port has been selected, or None if the active port has been
    /// disconnected..
    OutputPortSelected(Option<MidiPortDescriptor>),

    /// A MIDI message has arrived from external hardware.
    Midi(MidiChannel, MidiMessage),

    /// An error occurred.
    Error(MidiServiceError),

    /// The MIDI engine has successfully processed [MidiServiceInput::Quit], and
    /// the service will go away shortly.
    Quit,
}

/// [MidiService] error types.
#[derive(Clone, Debug)]
pub enum MidiServiceError {
    GeneralError,
    InError(MidiInServiceError),
    OutError(MidiOutServiceError),
}

/// Provides a crossbeam-channels interface to the
/// [midir](https://crates.io/crates/midir) crate.
#[derive(Debug)]
pub struct MidiService {
    inputs: CrossbeamChannel<MidiServiceInput>,
    events: CrossbeamChannel<MidiServiceEvent>,

    in_service: MidiInService,
    out_service: MidiOutService,
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
            in_service: Default::default(),
            out_service: Default::default(),
        };

        r.start_thread();
        r
    }

    fn start_thread(&self) {
        let receiver = self.inputs.receiver.clone();
        let sender = self.events.sender.clone();
        let in_receiver = self.in_service.events.receiver.clone();
        let in_sender = self.in_service.inputs.sender.clone();
        let out_receiver = self.out_service.events.receiver.clone();
        let out_sender = self.out_service.inputs.sender.clone();

        std::thread::spawn(move || {
            let mut sel = Select::default();

            let input_index = sel.recv(&receiver);
            let in_index = sel.recv(&in_receiver);
            let out_index = sel.recv(&out_receiver);

            loop {
                let oper = sel.select();
                match oper.index() {
                    index if index == input_index => {
                        if let Ok(input) = oper.recv(&receiver) {
                            match input {
                                MidiServiceInput::RefreshPorts => {
                                    let _ = in_sender.try_send(MidiInServiceInput::RefreshPorts);
                                    let _ = out_sender.try_send(MidiOutServiceInput::RefreshPorts);
                                }
                                MidiServiceInput::SelectInputPort(port) => {
                                    if let Some(port) = port {
                                        let _ =
                                            in_sender.try_send(MidiInServiceInput::Connect(port));
                                    } else {
                                        let _ = in_sender.try_send(MidiInServiceInput::Disconnect);
                                    }
                                }
                                MidiServiceInput::SelectOutputPort(port) => {
                                    if let Some(port) = port {
                                        let _ =
                                            out_sender.try_send(MidiOutServiceInput::Connect(port));
                                    } else {
                                        let _ =
                                            out_sender.try_send(MidiOutServiceInput::Disconnect);
                                    }
                                }
                                MidiServiceInput::Midi(channel, message) => {
                                    let _ = out_sender
                                        .try_send(MidiOutServiceInput::Midi(channel, message));
                                }
                                MidiServiceInput::Quit => {
                                    let _ = in_sender.try_send(MidiInServiceInput::Quit);
                                    let _ = out_sender.try_send(MidiOutServiceInput::Quit);
                                    let _ = sender.try_send(MidiServiceEvent::Quit);
                                }
                            }
                        }
                    }
                    index if index == in_index => {
                        if let Ok(event) = oper.recv(&in_receiver) {
                            match event {
                                MidiInServiceEvent::Ports(ports) => {
                                    let _ = sender.try_send(MidiServiceEvent::InputPorts(ports));
                                }
                                MidiInServiceEvent::Connected(port) => {
                                    let _ = sender
                                        .try_send(MidiServiceEvent::InputPortSelected(Some(port)));
                                }
                                MidiInServiceEvent::Disconnected => {
                                    let _ =
                                        sender.try_send(MidiServiceEvent::InputPortSelected(None));
                                }
                                MidiInServiceEvent::Midi(channel, message) => {
                                    let message =
                                        MidiUtils::substitute_note_off_for_note_on_vel_zero(
                                            message,
                                        );
                                    let _ =
                                        sender.try_send(MidiServiceEvent::Midi(channel, message));
                                }
                                MidiInServiceEvent::Error(e) => {
                                    let _ = sender.try_send(MidiServiceEvent::Error(
                                        MidiServiceError::InError(e),
                                    ));
                                }
                            }
                        }
                    }
                    index if index == out_index => {
                        if let Ok(event) = oper.recv(&out_receiver) {
                            match event {
                                MidiOutServiceEvent::Ports(ports) => {
                                    let _ = sender.try_send(MidiServiceEvent::OutputPorts(ports));
                                }
                                MidiOutServiceEvent::Connected(port) => {
                                    let _ = sender
                                        .try_send(MidiServiceEvent::OutputPortSelected(Some(port)));
                                }
                                MidiOutServiceEvent::Disconnected => {
                                    let _ =
                                        sender.try_send(MidiServiceEvent::OutputPortSelected(None));
                                }
                                MidiOutServiceEvent::Error(e) => {
                                    let _ = sender.try_send(MidiServiceEvent::Error(
                                        MidiServiceError::OutError(e),
                                    ));
                                }
                            }
                        }
                    }
                    _ => {
                        eprintln!("select returned unexpected index");
                    }
                }
            }
        });
    }
}
impl ProvidesService<MidiServiceInput, MidiServiceEvent> for MidiService {
    fn sender(&self) -> &crossbeam::channel::Sender<MidiServiceInput> {
        &self.inputs.sender
    }

    fn receiver(&self) -> &crossbeam::channel::Receiver<MidiServiceEvent> {
        &self.events.receiver
    }
}

#[derive(Clone, Debug)]
enum MidiInServiceInput {
    /// Requests a list of input ports that are available to connect to.
    RefreshPorts,
    /// Requests that the service connect to a port. The parameter is one of the
    /// values returned by the last [MidiInServiceEvent::Ports].
    ///
    /// Currently, only one connection at a time is allowed. Thus, connecting
    /// will disconnect any existing connection.
    Connect(MidiPortDescriptor),
    /// Requests that the service close the current connection, if any.
    Disconnect,
    /// Asks the service to quit.
    Quit,
}

#[derive(Clone, Debug)]
enum MidiInServiceEvent {
    /// Returns a list of available input ports.
    Ports(Vec<MidiPortDescriptor>),
    /// Indicates that the [MidiInServiceInput::Connect] request succeeded.
    Connected(MidiPortDescriptor),
    /// Indicates that the current connection has closed.
    Disconnected,
    /// A MIDI message has arrived from external hardware.
    Midi(MidiChannel, MidiMessage),
    /// Something went wrong.
    Error(MidiInServiceError),
}

/// [MidiInService] error types.
#[derive(Clone, Debug)]
pub enum MidiInServiceError {
    ConnectionFailed,
    InitFailed(String),
}

/// Wraps a [midir](https://crates.io/crates/midir) [MidiInput] with a
/// crossbeam-channels interface.
#[derive(Debug)]
struct MidiInService {
    inputs: CrossbeamChannel<MidiInServiceInput>,
    events: CrossbeamChannel<MidiInServiceEvent>,
}
impl Default for MidiInService {
    fn default() -> Self {
        Self::new()
    }
}
impl MidiInService {
    const CLIENT_NAME: &'static str = "Ensnare MIDI input";

    pub fn new() -> Self {
        let r = Self {
            inputs: Default::default(),
            events: Default::default(),
        };

        r.start_thread();
        r
    }

    fn refresh_ports_and_descriptors(
        midir_input: &Option<MidiInput>,
    ) -> (Vec<MidiInputPort>, Vec<MidiPortDescriptor>) {
        let ports = if let Some(m) = midir_input.as_ref() {
            m.ports().clone()
        } else {
            Default::default()
        };
        let descriptors = if let Some(mi) = midir_input.as_ref() {
            ports
                .iter()
                .enumerate()
                .map(|(i, port)| MidiPortDescriptor::new_with(i, mi.port_name(port).ok()))
                .collect()
        } else {
            Default::default()
        };
        (ports, descriptors)
    }

    fn start_thread(&self) {
        let receiver = self.inputs.receiver.clone();
        let sender = self.events.sender.clone();
        std::thread::spawn(move || {
            let mut midir = match MidiInput::new(Self::CLIENT_NAME) {
                Ok(midir) => Some(midir),
                Err(e) => {
                    let _ = sender.try_send(MidiInServiceEvent::Error(
                        MidiInServiceError::InitFailed(e.to_string()),
                    ));
                    None
                }
            };
            let mut connection: Option<MidiInputConnection<()>> = None;
            let (mut ports, mut port_descriptors) = Self::refresh_ports_and_descriptors(&midir);

            // Send the ports after init so caller doesn't need to ask.
            let _ = sender.try_send(MidiInServiceEvent::Ports(port_descriptors));

            while let Ok(input) = receiver.recv() {
                match input {
                    MidiInServiceInput::RefreshPorts => {
                        (ports, port_descriptors) = Self::refresh_ports_and_descriptors(&midir);
                        let _ = sender.try_send(MidiInServiceEvent::Ports(port_descriptors));
                    }
                    MidiInServiceInput::Connect(port_descriptor) => {
                        if let Some(m) = midir.take() {
                            // If there is an active connection, we should close
                            // it.
                            if let Some(c) = connection.take() {
                                c.close();
                            }

                            // Now check to see which port we need to connect
                            // to.
                            let index = port_descriptor.index;
                            if index < ports.len() {
                                let port = &ports[index];
                                let sender_clone = sender.clone();

                                // Is this really the same port?
                                if Ok(port_descriptor.name.clone()) == m.port_name(port) {
                                    // Yes. Connect.
                                    connection = m
                                        .connect(
                                            port,
                                            &format!("Ensnare: input {}", port_descriptor.name),
                                            move |_, event, _| {
                                                if let Ok(LiveEvent::Midi { channel, message }) =
                                                    LiveEvent::parse(event)
                                                {
                                                    let _ = sender_clone.try_send(
                                                        MidiInServiceEvent::Midi(
                                                            MidiChannel::from(channel),
                                                            message,
                                                        ),
                                                    );
                                                }
                                            },
                                            (),
                                        )
                                        .ok();
                                    if connection.is_some() {
                                        let _ = sender.try_send(MidiInServiceEvent::Connected(
                                            port_descriptor,
                                        ));
                                    } else {
                                        let _ = sender.try_send(MidiInServiceEvent::Error(
                                            MidiInServiceError::ConnectionFailed,
                                        ));
                                    }
                                }
                            } else {
                                eprintln!(
                                    "error - MidiInServiceInput::Connect descriptor didn't match"
                                );
                            }

                            // The current MidiInput has been taken. We always
                            // want a MidiInput, even if the connection took the
                            // active one, because we need one to enumerate
                            // ports.
                            midir = MidiInput::new(Self::CLIENT_NAME).ok();
                        }
                    }
                    MidiInServiceInput::Disconnect => {
                        if let Some(c) = connection.take() {
                            c.close();
                            let _ = sender.try_send(MidiInServiceEvent::Disconnected);
                        }
                    }
                    MidiInServiceInput::Quit => break,
                }
            }
        });
    }
}
impl ProvidesService<MidiInServiceInput, MidiInServiceEvent> for MidiInService {
    fn sender(&self) -> &crossbeam::channel::Sender<MidiInServiceInput> {
        &self.inputs.sender
    }

    fn receiver(&self) -> &crossbeam::channel::Receiver<MidiInServiceEvent> {
        &self.events.receiver
    }
}

#[derive(Clone, Debug)]
enum MidiOutServiceInput {
    /// Requests a list of output ports that are available to connect to.
    RefreshPorts,
    /// Requests that the service connect to a port. The parameter is one of the
    /// values returned by the last [MidiOutServiceEvent::Ports].
    ///
    /// Currently, only one connection at a time is allowed. Thus, connecting
    /// will disconnect any existing connection.
    Connect(MidiPortDescriptor),
    /// Requests that the service close the current connection, if any.
    Disconnect,
    /// A MIDI message should be sent to external hardware.
    Midi(MidiChannel, MidiMessage),
    /// Asks the service to quit.
    Quit,
}

#[derive(Clone, Debug)]
enum MidiOutServiceEvent {
    /// Returns a list of available output ports.
    Ports(Vec<MidiPortDescriptor>),
    /// Indicates that the [MidiOutServiceInput::Connect] request succeeded.
    Connected(MidiPortDescriptor),
    /// Indicates that the current connection has closed.
    Disconnected,
    /// Something went wrong.
    Error(MidiOutServiceError),
}

/// [MidiOutService] error types.
#[derive(Clone, Debug)]
pub enum MidiOutServiceError {
    ConnectionFailed,
    InitFailed(String),
}

/// Wraps a [midir](https://crates.io/crates/midir) [MidiOutput] with a
/// crossbeam-channels interface.
#[derive(Debug)]
struct MidiOutService {
    inputs: CrossbeamChannel<MidiOutServiceInput>,
    events: CrossbeamChannel<MidiOutServiceEvent>,
}
impl Default for MidiOutService {
    fn default() -> Self {
        Self::new()
    }
}
impl MidiOutService {
    const CLIENT_NAME: &'static str = "Ensnare MIDI output";

    pub fn new() -> Self {
        let r = Self {
            inputs: Default::default(),
            events: Default::default(),
        };

        r.start_thread();
        r
    }

    fn refresh_ports_and_descriptors(
        midir_input: &Option<MidiOutput>,
    ) -> (Vec<MidiOutputPort>, Vec<MidiPortDescriptor>) {
        let ports = if let Some(m) = midir_input.as_ref() {
            m.ports().clone()
        } else {
            Default::default()
        };
        let descriptors = if let Some(mi) = midir_input.as_ref() {
            ports
                .iter()
                .enumerate()
                .map(|(i, port)| MidiPortDescriptor::new_with(i, mi.port_name(port).ok()))
                .collect()
        } else {
            Default::default()
        };
        (ports, descriptors)
    }

    fn start_thread(&self) {
        let receiver = self.inputs.receiver.clone();
        let sender = self.events.sender.clone();
        std::thread::spawn(move || {
            let mut midir = match MidiOutput::new(Self::CLIENT_NAME) {
                Ok(midir) => Some(midir),
                Err(e) => {
                    let _ = sender.try_send(MidiOutServiceEvent::Error(
                        MidiOutServiceError::InitFailed(e.to_string()),
                    ));
                    None
                }
            };
            let mut connection: Option<MidiOutputConnection> = None;
            let (mut ports, mut port_descriptors) = Self::refresh_ports_and_descriptors(&midir);

            // Send the ports after init so caller doesn't need to ask.
            let _ = sender.try_send(MidiOutServiceEvent::Ports(port_descriptors));

            while let Ok(input) = receiver.recv() {
                match input {
                    MidiOutServiceInput::RefreshPorts => {
                        (ports, port_descriptors) = Self::refresh_ports_and_descriptors(&midir);
                        let _ = sender.try_send(MidiOutServiceEvent::Ports(port_descriptors));
                    }
                    MidiOutServiceInput::Connect(port_descriptor) => {
                        if let Some(m) = midir.take() {
                            // If there is an active connection, we should close
                            // it.
                            if let Some(c) = connection.take() {
                                c.close();
                            }

                            // Now check to see which port we need to connect
                            // to.
                            let index = port_descriptor.index;
                            if index < ports.len() {
                                let port = &ports[index];

                                // Is this really the same port?
                                if Ok(port_descriptor.name.clone()) == m.port_name(port) {
                                    // Yes. Connect.
                                    connection = m.connect(port, &port_descriptor.name).ok();
                                    if connection.is_some() {
                                        let _ = sender.try_send(MidiOutServiceEvent::Connected(
                                            port_descriptor,
                                        ));
                                    } else {
                                        let _ = sender.try_send(MidiOutServiceEvent::Error(
                                            MidiOutServiceError::ConnectionFailed,
                                        ));
                                    }
                                }
                            } else {
                                eprintln!(
                                    "error - MidiInServiceInput::Connect descriptor didn't match"
                                );
                            }

                            // The current MidiOutput has been taken. We always
                            // want a MidiOutput, even if the connection took
                            // the active one, because we need one to enumerate
                            // ports.
                            midir = MidiOutput::new(Self::CLIENT_NAME).ok();
                        }
                    }
                    MidiOutServiceInput::Disconnect => {
                        if let Some(c) = connection.take() {
                            c.close();
                            let _ = sender.try_send(MidiOutServiceEvent::Disconnected);
                        }
                    }
                    MidiOutServiceInput::Midi(channel, message) => {
                        if let Some(c) = connection.as_mut() {
                            let event = LiveEvent::Midi {
                                channel: u4::from(channel.0),
                                message,
                            };
                            let mut buffer = Vec::new();
                            if let Ok(_) = event.write(&mut buffer) {
                                let _ = c.send(&buffer);
                            }
                        }
                    }
                    MidiOutServiceInput::Quit => break,
                }
            }
        });
    }
}
impl ProvidesService<MidiOutServiceInput, MidiOutServiceEvent> for MidiOutService {
    fn sender(&self) -> &crossbeam::channel::Sender<MidiOutServiceInput> {
        &self.inputs.sender
    }

    fn receiver(&self) -> &crossbeam::channel::Receiver<MidiOutServiceEvent> {
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
