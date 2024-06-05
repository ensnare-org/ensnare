// Copyright (c) 2024 Mike Tsao

use ensnare::prelude::*;
use ensnare_services::prelude::*;

fn main() -> anyhow::Result<()> {
    // Instantiate the service.
    //
    // Note that we need to keep the service in scope even though we don't
    // interact with it after setup. If we don't keep the reference, it will be
    // dropped.
    let service = MidiService::default();
    let sender = service.sender().clone();
    let receiver = service.receiver().clone();

    let _ = sender.send(MidiServiceInput::RefreshPorts);

    while let Ok(input) = receiver.recv() {
        match input {
            MidiServiceEvent::InputPorts(ports) => {
                println!("Input ports: {ports:?}");

                // The first port is usually a null port, and the second one is
                // the first real one.
                if ports.len() > 0 {
                    let _ = sender.send(MidiServiceInput::SelectInputPort(Some(ports[1].clone())));
                }
            }
            MidiServiceEvent::InputPortSelected(port) => {
                println!("Successfully connected to input {port:?}. Generate a note-off event on Middle C (MIDI note 60) to quit.");
            }
            MidiServiceEvent::OutputPorts(ports) => {
                println!("Output ports: {ports:?}");
                if ports.len() > 0 {
                    let _ = sender.send(MidiServiceInput::SelectOutputPort(Some(ports[1].clone())));
                }
            }
            MidiServiceEvent::OutputPortSelected(port) => {
                println!("Successfully connected to output {port:?}.");

                for i in 0..5 {
                    let _ = sender.send(MidiServiceInput::Midi(
                        MidiChannel::default(),
                        MidiMessage::NoteOn {
                            key: u7::from(60 + i),
                            vel: 127.into(),
                        },
                    ));
                }
            }
            MidiServiceEvent::Midi(channel, message) => {
                println!("Midi: {channel} {message:?}");

                #[allow(unused_variables)]
                if let MidiMessage::NoteOff { key, vel } = message {
                    if key == 60 {
                        // All notes off, just in case
                        let _ = sender.send(MidiServiceInput::Midi(
                            MidiChannel::default(),
                            MidiMessage::Controller {
                                controller: 123.into(),
                                value: 0.into(),
                            },
                        ));
                        let _ = sender.send(MidiServiceInput::Quit);
                        break;
                    }
                }
            }
            MidiServiceEvent::Quit => todo!(),
            MidiServiceEvent::Error(e) => {
                eprintln!("error: {e:?}")
            }
        }
    }
    println!("exiting...");

    Ok(())
}
