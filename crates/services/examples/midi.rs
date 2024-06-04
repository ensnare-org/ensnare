// Copyright (c) 2024 Mike Tsao

use ensnare::traits::ProvidesService;
use ensnare_services::prelude::*;

fn main() -> anyhow::Result<()> {
    // Instantiate the service.
    //
    // Note that we need to keep the service in scope even though we don't
    // interact with it after setup. If we don't keep the reference, it will be
    // dropped.
    let audio_service = MidiService::default();
    let _sender = audio_service.sender().clone();
    let receiver = audio_service.receiver().clone();

    while let Ok(input) = receiver.recv() {
        match input {
            MidiServiceEvent::InputPorts(_) => todo!(),
            MidiServiceEvent::InputPortSelected(_) => todo!(),
            MidiServiceEvent::OutputPorts(_) => todo!(),
            MidiServiceEvent::OutputPortSelected(_) => todo!(),
            MidiServiceEvent::Midi(_, _) => todo!(),
            MidiServiceEvent::MidiOut => todo!(),
            MidiServiceEvent::Quit => todo!(),
        }
    }

    Ok(())
}
