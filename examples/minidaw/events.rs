// Copyright (c) 2024 Mike Tsao

use crate::settings::SettingsEvent;
use crossbeam::channel::{Receiver, Select, Sender};
use ensnare::prelude::*;
use ensnare_services::{prelude::*, ProjectService, ProjectServiceEvent, ProjectServiceInput};
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub(super) enum LoadError {
    #[error("see https://crates.io/crates/thiserror to write better error messages")]
    Todo,
}

#[allow(dead_code)]
#[derive(Debug, derive_more::Display)]
pub(super) enum SaveError {
    Todo,
}

#[derive(Debug)]
pub(super) enum MiniDawInput {
    Quit,
}

/// An aggregation of all the service events that the app might want to process.
#[derive(Debug)]
pub(super) enum MiniDawEvent {
    MidiPanelEvent(MidiServiceEvent),
    CpalAudioServiceEvent(CpalAudioServiceEvent),
    ProjectServiceEvent(ProjectServiceEvent),
    Quit,
}

#[derive(Debug)]
pub(super) struct MiniDawEventAggregationService {
    inputs: CrossbeamChannel<MiniDawInput>,
    events: CrossbeamChannel<MiniDawEvent>,

    // The aggregated services. Avoid speaking directly to them; use the
    // channels instead.
    audio_service: CpalAudioService,
    midi_service: MidiService,
    project_service: ProjectService,

    settings_receiver: Receiver<SettingsEvent>,
}
impl MiniDawEventAggregationService {
    pub fn new_with(
        audio_service: CpalAudioService,
        midi_service: MidiService,
        project_service: ProjectService,
        settings_receiver: &Receiver<SettingsEvent>,
    ) -> Self {
        let r = Self {
            inputs: Default::default(),
            events: Default::default(),
            audio_service,
            midi_service,
            project_service,
            settings_receiver: settings_receiver.clone(),
        };
        r.spawn_thread();
        r
    }

    /// Watches all the channel receivers we know about, and either handles them
    /// immediately off the UI thread or forwards them to the app's event
    /// channel.
    fn spawn_thread(&self) {
        // Sends aggregated events for the app to handle.
        let app_sender = self.events.sender.clone();

        // Takes commands from the app.
        let app_receiver = self.inputs.receiver.clone();

        // Each of these pairs communicates with a service.
        let audio_sender = self.audio_service.sender().clone();
        let audio_receiver = self.audio_service.receiver().clone();

        let midi_sender = self.midi_service.sender().clone();
        let _midi_receiver = self.midi_service.receiver().clone();

        let midi_receiver = self.midi_service.receiver().clone();
        let project_sender = self.project_service.sender().clone();
        let project_receiver = self.project_service.receiver().clone();
        let settings_receiver = self.settings_receiver.clone();

        let _ = std::thread::spawn(move || {
            let mut sel = Select::new();
            let app_index = sel.recv(&app_receiver);
            let midi_index = sel.recv(&midi_receiver);
            let audio_index = sel.recv(&audio_receiver);
            let project_index = sel.recv(&project_receiver);
            let settings_index = sel.recv(&settings_receiver);
            let mut should_route_midi = true;

            loop {
                let operation = sel.select();
                match operation.index() {
                    index if index == app_index => {
                        if let Ok(input) = operation.recv(&app_receiver) {
                            match input {
                                MiniDawInput::Quit => {
                                    let _ = audio_sender.send(CpalAudioServiceInput::Quit);
                                    let _ = midi_sender.send(MidiServiceInput::Quit);
                                    let _ = project_sender.send(ProjectServiceInput::ServiceQuit);
                                    let _ = app_sender.send(MiniDawEvent::Quit);
                                    return;
                                }
                            }
                        }
                    }
                    index if index == audio_index => {
                        if let Ok(event) = operation.recv(&audio_receiver) {
                            match event {
                                CpalAudioServiceEvent::FramesNeeded(count) => {
                                    let _ = project_sender
                                        .send(ProjectServiceInput::FramesNeeded(count));
                                }
                                CpalAudioServiceEvent::Reset(sample_rate, channel_count) => {
                                    let _ = project_sender.send(ProjectServiceInput::AudioReset(
                                        SampleRate(sample_rate),
                                        channel_count,
                                    ));
                                }
                                CpalAudioServiceEvent::Underrun => {}
                            }
                            let _ = app_sender.send(MiniDawEvent::CpalAudioServiceEvent(event));
                        }
                    }
                    index if index == midi_index => {
                        if let Ok(event) = operation.recv(&midi_receiver) {
                            match event {
                                // MIDI messages that came from external interfaces.
                                MidiServiceEvent::Midi(channel, message) => {
                                    // Forward right away to the project. We
                                    // still forward it to the app so that it
                                    // can update the UI activity indicators.
                                    let _ = project_sender
                                        .send(ProjectServiceInput::Midi(channel, message));
                                }
                                _ => {
                                    // fall through and forward to the app.
                                }
                            }
                            let _ = app_sender.send(MiniDawEvent::MidiPanelEvent(event));
                        }
                    }
                    index if index == project_index => {
                        if let Ok(event) = operation.recv(&project_receiver) {
                            match event {
                                // MIDI messages that came from the project.
                                ProjectServiceEvent::Midi(channel, message) => {
                                    if should_route_midi {
                                        // Fast-route generated MIDI messages so app
                                        // doesn't have to. This handles
                                        // ProjectServiceEvent::Midi, so the app
                                        // should never see it.
                                        let _ = midi_sender
                                            .send(MidiServiceInput::Midi(channel, message));
                                    }
                                }
                                _ => {
                                    let _ =
                                        app_sender.send(MiniDawEvent::ProjectServiceEvent(event));
                                }
                            }
                        }
                    }
                    index if index == settings_index => {
                        if let Ok(event) = operation.recv(&settings_receiver) {
                            match event {
                                SettingsEvent::ShouldRouteExternally(should) => {
                                    should_route_midi = should;
                                }
                            }
                        }
                    }
                    _ => {
                        panic!("missing case for a new receiver")
                    }
                }
            }
        });
    }

    pub fn sender(&self) -> &Sender<MiniDawInput> {
        &self.inputs.sender
    }

    pub fn receiver(&self) -> &Receiver<MiniDawEvent> {
        &self.events.receiver
    }
}
