// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use crate::types::{AudioQueue, VisualizationQueue};
use anyhow::Error;
use crossbeam_channel::{Receiver, Sender};
#[cfg(feature = "egui")]
use eframe::egui::Key;
#[cfg(feature = "egui")]
use egui::KeyHandler;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub enum ProjectServiceInput {
    AudioQueue(AudioQueue),
    #[cfg(feature = "egui")]
    KeyEvent(Key, bool, Option<Key>),
    Midi(MidiChannel, MidiMessage),
    NeedsAudio(usize),
    NextTimelineDisplayer,
    ProjectExportToWav(Option<PathBuf>),
    ProjectLinkControl(Uid, Uid, ControlIndex),
    ProjectLoad(PathBuf),
    ProjectNew,
    ProjectPlay,
    ProjectRemoveEntity(Uid),
    ProjectSave(Option<PathBuf>),
    ProjectSetSampleRate(SampleRate),
    ProjectStop,
    ServiceInit,
    ServiceQuit,
    TrackAddEntity(TrackUid, EntityKey),
    TrackNewAudio,
    TrackNewAux,
    TrackNewMidi,
    VisualizationQueue(VisualizationQueue),
}

#[derive(Debug)]
pub enum ProjectServiceEvent {
    ExportFailed(Error),
    Exported(PathBuf),
    IsPerformingChanged(bool),
    LoadFailed(PathBuf, Error),
    Loaded(Arc<RwLock<Project>>), // The supplied Project is for the recipient to keep. No need to Arc::clone().
    Midi(MidiChannel, MidiMessage), // Handled by EnsnareEventAggregationService, never sent to app.
    Quit,
    SaveFailed(Error),
    Saved(PathBuf),
    TitleChanged(ProjectTitle),
}

/// A wrapper around a [Project] that provides a channel-based interface to it.
#[derive(Debug)]
pub struct ProjectService {
    input_channels: ChannelPair<ProjectServiceInput>,
    event_channels: ChannelPair<ProjectServiceEvent>,

    factory: Arc<EntityFactory<dyn EntityBounds>>,
}
impl ProjectService {
    pub fn new_with(factory: &Arc<EntityFactory<dyn EntityBounds>>) -> Self {
        let r = Self {
            input_channels: Default::default(),
            event_channels: Default::default(),
            factory: Arc::clone(factory),
        };
        r.spawn_thread();
        let _ = r.sender().send(ProjectServiceInput::ServiceInit);
        r
    }

    fn spawn_thread(&self) {
        let receiver = self.input_channels.receiver.clone();
        let sender = self.event_channels.sender.clone();
        let factory = Arc::clone(&self.factory);
        std::thread::spawn(move || {
            let mut daemon = ProjectServiceDaemon::new_with(receiver, sender, factory);
            daemon.execute();
        });
    }

    /// The receive side of the [ProjectServiceEvent] channel.
    pub fn receiver(&self) -> &Receiver<ProjectServiceEvent> {
        &self.event_channels.receiver
    }

    /// The sender side of the [ProjectServiceInput] channel.
    pub fn sender(&self) -> &Sender<ProjectServiceInput> {
        &self.input_channels.sender
    }
}

struct ProjectServiceDaemon {
    receiver: Receiver<ProjectServiceInput>,
    sender: Sender<ProjectServiceEvent>,
    factory: Arc<EntityFactory<dyn EntityBounds>>,

    project: Arc<RwLock<Project>>,

    #[cfg(feature = "egui")]
    key_handler: KeyHandler,

    audio_queue: Option<AudioQueue>,
    visualization_queue: Option<VisualizationQueue>,
}
impl ProjectServiceDaemon {
    pub fn new_with(
        receiver: Receiver<ProjectServiceInput>,
        sender: Sender<ProjectServiceEvent>,
        factory: Arc<EntityFactory<dyn EntityBounds>>,
    ) -> Self {
        Self {
            receiver,
            sender,
            factory,
            project: Arc::new(RwLock::new(Project::new_project())),
            #[cfg(feature = "egui")]
            key_handler: Default::default(),
            audio_queue: Default::default(),
            visualization_queue: Default::default(),
        }
    }

    fn notify_new_project(&self) {
        let _ = self
            .sender
            .send(ProjectServiceEvent::Loaded(Arc::clone(&self.project)));
    }

    fn set_up_new_project(&self, new_project: &mut Project) {
        if let Some(queue) = self.audio_queue.as_ref() {
            new_project.e.audio_queue = Some(Arc::clone(queue));
        }
        if let Some(queue) = self.visualization_queue.as_ref() {
            new_project.e.visualization_queue = Some(queue.clone());
        }
    }

    fn swap_project(&mut self, mut new_project: Project) {
        self.set_up_new_project(&mut new_project);
        self.project = Arc::new(RwLock::new(new_project));
        self.notify_new_project();
    }

    fn execute(&mut self) {
        while let Ok(input) = self.receiver.recv() {
            match input {
                ProjectServiceInput::ServiceInit => {
                    self.notify_new_project();
                }
                ProjectServiceInput::ProjectNew => {
                    // TODO: set_up_successor
                    let new_project = Project::new_project();
                    self.swap_project(new_project);
                }
                ProjectServiceInput::ProjectLoad(path) => match Project::load(path.clone()) {
                    Ok(new_project) => {
                        self.swap_project(new_project);
                    }
                    Err(e) => {
                        let _ = self.sender.send(ProjectServiceEvent::LoadFailed(path, e));
                    }
                },
                ProjectServiceInput::ProjectSave(path) => {
                    let mut project = self.project.write().unwrap();
                    match project.save(path) {
                        Ok(save_path) => {
                            let _ = self.sender.send(ProjectServiceEvent::Saved(save_path));
                        }
                        Err(e) => {
                            let _ = self.sender.send(ProjectServiceEvent::SaveFailed(e));
                        }
                    }
                }
                ProjectServiceInput::ServiceQuit => {
                    eprintln!("ProjectServiceInput::Quit");
                    let _ = self.sender.send(ProjectServiceEvent::Quit);
                    break;
                }
                ProjectServiceInput::ProjectSetSampleRate(sample_rate) => {
                    self.project
                        .write()
                        .unwrap()
                        .update_sample_rate(sample_rate);
                }
                ProjectServiceInput::ProjectPlay => {
                    self.project.write().unwrap().play();
                    let _ = self
                        .sender
                        .send(ProjectServiceEvent::IsPerformingChanged(true));
                }
                ProjectServiceInput::ProjectStop => {
                    self.project.write().unwrap().stop();
                    let _ = self
                        .sender
                        .send(ProjectServiceEvent::IsPerformingChanged(false));
                }
                ProjectServiceInput::TrackAddEntity(track_uid, key) => {
                    if let Ok(mut project) = self.project.write() {
                        let uid = project.mint_entity_uid();
                        if let Some(entity) = self.factory.new_entity(&key, uid) {
                            let _ = project.add_entity(track_uid, entity, Some(uid));
                            let _ = project
                                .set_midi_receiver_channel(uid, Some(MidiChannel::default()));
                        } else {
                            eprintln!("ProjectServiceInput::TrackAddEntity failed");
                        }
                    }
                }
                ProjectServiceInput::ProjectLinkControl(source_uid, target_uid, index) => {
                    let _ = self
                        .project
                        .write()
                        .unwrap()
                        .link(source_uid, target_uid, index);
                }
                #[cfg(feature = "egui")]
                ProjectServiceInput::KeyEvent(key, pressed, _physical_key) => {
                    if let Some(message) = self.key_handler.handle_key(&key, pressed) {
                        self.project.write().unwrap().handle_midi_message(
                            MidiChannel::default(),
                            message,
                            &mut |c, m| {
                                eprintln!("TODO: {c:?} {m:?}");
                            },
                        )
                    }
                }
                ProjectServiceInput::NextTimelineDisplayer => {
                    if let Ok(mut project) = self.project.write() {
                        let selected_track_uids = project.view_state.track_selection_set.clone();
                        selected_track_uids
                            .iter()
                            .for_each(|track_uid| project.advance_track_view_mode(*track_uid));
                    }
                }
                ProjectServiceInput::AudioQueue(queue) => {
                    self.audio_queue = Some(Arc::clone(&queue));
                    self.project.write().unwrap().e.audio_queue = Some(queue);
                }
                ProjectServiceInput::VisualizationQueue(queue) => {
                    self.visualization_queue = Some(queue.clone());
                    self.project.write().unwrap().e.visualization_queue = Some(queue)
                }
                ProjectServiceInput::NeedsAudio(count) => {
                    self.project.write().unwrap().fill_audio_queue(
                        count,
                        Some(&mut |c, m| {
                            // If we had a channel sender to the MIDI service,
                            // then we could send directly there from here. But
                            // that would introduce a dependency between
                            // ProjectService and MidiService, and I'd rather
                            // stay with a simple hub/spoke event architecture
                            // until it proves to be a performance issue.
                            let _ = self.sender.send(ProjectServiceEvent::Midi(c, m));
                        }),
                    );
                }
                ProjectServiceInput::Midi(channel, message) => self
                    .project
                    .write()
                    .unwrap()
                    .handle_midi_message(channel, message, &mut |c, m| {
                        eprintln!("TODO: {c:?} {m:?}");
                    }),
                ProjectServiceInput::ProjectRemoveEntity(uid) => {
                    let _ = self.project.write().unwrap().remove_entity(uid);
                }
                ProjectServiceInput::TrackNewAudio => {
                    let _ = self.project.write().unwrap().new_audio_track();
                }
                ProjectServiceInput::TrackNewAux => {
                    let _ = self.project.write().unwrap().new_aux_track();
                }
                ProjectServiceInput::TrackNewMidi => {
                    let _ = self.project.write().unwrap().new_midi_track();
                }
                ProjectServiceInput::ProjectExportToWav(path) => {
                    let path = path.unwrap_or(PathBuf::from("exported-project.wav"));
                    let _ = self.project.write().unwrap().export_to_wav(path);
                }
            }
        }
        eprintln!("ProjectServiceDaemon exit");
    }
}

#[cfg(feature = "egui")]
pub(super) mod egui {
    use super::*;
    use derivative::Derivative;

    /// Represents an octave as MIDI conventions expect them: A before middle C is
    /// in octave 5, and the range is from 0 to 10.
    ///
    /// TODO: I looked around for a bounded integer type or crate, but all made a
    /// mountain out of this molehill-sized use case.
    #[derive(Debug, Derivative)]
    #[derivative(Default)]
    pub(super) struct Octave(#[derivative(Default(value = "5"))] pub u8);
    impl Octave {
        fn decrease(&mut self) {
            if self.0 > 0 {
                self.0 -= 1;
            }
        }
        fn increase(&mut self) {
            if self.0 < 10 {
                self.0 += 1;
            }
        }
    }

    /// Maps [eframe::egui::Key] presses to MIDI events using a piano-keyboard-like
    /// layout of QWERTY keys homed at the A-K row. Contains a bit of state, using
    /// left/right arrow to change octaves.
    #[derive(Debug, Default)]
    pub(super) struct KeyHandler {
        octave: Octave,
    }

    impl KeyHandler {
        pub fn handle_key(&mut self, key: &Key, pressed: bool) -> Option<MidiMessage> {
            match key {
                Key::A => Some(self.midi_note_message(0, pressed)),
                Key::W => Some(self.midi_note_message(1, pressed)),
                Key::S => Some(self.midi_note_message(2, pressed)),
                Key::E => Some(self.midi_note_message(3, pressed)),
                Key::D => Some(self.midi_note_message(4, pressed)),
                Key::F => Some(self.midi_note_message(5, pressed)),
                Key::T => Some(self.midi_note_message(6, pressed)),
                Key::G => Some(self.midi_note_message(7, pressed)),
                Key::Y => Some(self.midi_note_message(8, pressed)),
                Key::H => Some(self.midi_note_message(9, pressed)),
                Key::U => Some(self.midi_note_message(10, pressed)),
                Key::J => Some(self.midi_note_message(11, pressed)),
                Key::K => Some(self.midi_note_message(12, pressed)),
                Key::O => Some(self.midi_note_message(13, pressed)),
                Key::ArrowLeft => {
                    if pressed {
                        self.octave.decrease();
                    }
                    None
                }
                Key::ArrowRight => {
                    if pressed {
                        self.octave.increase();
                    }
                    None
                }
                _ => None,
            }
        }

        fn midi_note_message(&self, midi_note_number: u8, pressed: bool) -> MidiMessage {
            let midi_note_number = (midi_note_number + self.octave.0 * 12).min(127);

            if pressed {
                MidiMessage::NoteOn {
                    key: u7::from(midi_note_number),
                    vel: u7::from(127),
                }
            } else {
                MidiMessage::NoteOff {
                    key: u7::from(midi_note_number),
                    vel: u7::from(0),
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn expected_messages_for_keystrokes() {
            let mut k = KeyHandler::default();
            let message = k.handle_key(&Key::A, true).unwrap();
            assert_eq!(
                message,
                MidiMessage::NoteOn {
                    key: u7::from(MidiNote::C4 as u8),
                    vel: u7::from(127)
                }
            );
        }

        #[test]
        fn octaves() {
            let mut k = KeyHandler::default();

            // Play a note at initial octave 4.
            let message = k.handle_key(&Key::A, true).unwrap();
            let _ = k.handle_key(&Key::A, false).unwrap();
            assert_eq!(
                message,
                MidiMessage::NoteOn {
                    key: u7::from(MidiNote::C4 as u8),
                    vel: u7::from(127)
                }
            );

            // Increase octave and try again.
            let _ = k.handle_key(&Key::ArrowRight, true);
            let message = k.handle_key(&Key::A, true).unwrap();
            let _ = k.handle_key(&Key::A, false).unwrap();
            assert_eq!(
                message,
                MidiMessage::NoteOn {
                    key: u7::from(MidiNote::C5 as u8),
                    vel: u7::from(127)
                }
            );

            // Up to maximum octave 10 (AKA octave 9).
            let _ = k.handle_key(&Key::ArrowRight, true);
            let _ = k.handle_key(&Key::ArrowRight, true);
            let _ = k.handle_key(&Key::ArrowRight, true);
            let _ = k.handle_key(&Key::ArrowRight, true);
            let message = k.handle_key(&Key::A, true).unwrap();
            let _ = k.handle_key(&Key::A, false).unwrap();
            assert_eq!(
                message,
                MidiMessage::NoteOn {
                    key: u7::from(MidiNote::C9 as u8),
                    vel: u7::from(127)
                }
            );

            let _ = k.handle_key(&Key::ArrowRight, true);
            let message = k.handle_key(&Key::A, true).unwrap();
            let _ = k.handle_key(&Key::A, false).unwrap();
            assert_eq!(
                message,
                MidiMessage::NoteOn {
                    key: u7::from(MidiNote::C9 as u8),
                    vel: u7::from(127)
                },
                "Trying to go higher than max octave shouldn't change anything."
            );

            // Now start over and try again with lower octaves.
            let mut k = KeyHandler::default();
            let _ = k.handle_key(&Key::ArrowLeft, true);
            let message = k.handle_key(&Key::A, true).unwrap();
            let _ = k.handle_key(&Key::A, false).unwrap();
            assert_eq!(
                message,
                MidiMessage::NoteOn {
                    key: u7::from(MidiNote::C3 as u8),
                    vel: u7::from(127)
                }
            );
            let _ = k.handle_key(&Key::ArrowLeft, true);
            let message = k.handle_key(&Key::A, true).unwrap();
            let _ = k.handle_key(&Key::A, false).unwrap();
            assert_eq!(
                message,
                MidiMessage::NoteOn {
                    key: u7::from(MidiNote::C2 as u8),
                    vel: u7::from(127)
                }
            );
            let _ = k.handle_key(&Key::ArrowLeft, true);
            let message = k.handle_key(&Key::A, true).unwrap();
            let _ = k.handle_key(&Key::A, false).unwrap();
            assert_eq!(
                message,
                MidiMessage::NoteOn {
                    key: u7::from(MidiNote::C1 as u8),
                    vel: u7::from(127)
                }
            );
            let _ = k.handle_key(&Key::ArrowLeft, true);
            let message = k.handle_key(&Key::A, true).unwrap();
            let _ = k.handle_key(&Key::A, false).unwrap();
            assert_eq!(
                message,
                MidiMessage::NoteOn {
                    key: u7::from(MidiNote::C0 as u8),
                    vel: u7::from(127)
                }
            );
            let _ = k.handle_key(&Key::ArrowLeft, true);
            let message = k.handle_key(&Key::A, true).unwrap();
            let _ = k.handle_key(&Key::A, false).unwrap();
            assert_eq!(
                message,
                MidiMessage::NoteOn {
                    key: u7::from(MidiNote::CSub0 as u8),
                    vel: u7::from(127)
                }
            );
            let _ = k.handle_key(&Key::ArrowLeft, true);
            let message = k.handle_key(&Key::A, true).unwrap();
            let _ = k.handle_key(&Key::A, false).unwrap();
            assert_eq!(
                message,
                MidiMessage::NoteOn {
                    key: u7::from(MidiNote::CSub0 as u8),
                    vel: u7::from(127)
                },
                "Trying to go below the lowest octave should stay at lowest octave."
            );
        }
    }
}
