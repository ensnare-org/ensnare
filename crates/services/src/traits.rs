// Copyright (c) 2024 Mike Tsao

//! Traits used by services.

use crossbeam::channel::{Receiver, Sender};

/// Service methods.
///
/// A service is something that usually runs in its own thread as a daemon and
/// that communicates with clients by crossbeam channels. It accepts Inputs and
/// produces Events.
pub trait ProvidesService<I: core::fmt::Debug, E: core::fmt::Debug> {
    /// The sender side of the Input channel. Use this to send commands to the
    /// service.
    fn sender(&self) -> &Sender<I>;

    /// A convenience method to send Inputs to the service. Calling this implies
    /// that the caller has kept a reference to the service, which is uncommon,
    /// as the main value of services is to be able to clone senders with
    /// reckless abandon.
    fn send_input(&self, input: I) {
        if let Err(e) = self.sender().try_send(input) {
            eprintln!("While sending: {e:?}");
        }
    }

    /// The receiver side of the Event channel. Integrate this into a listener
    /// loop to respond to events.
    fn receiver(&self) -> &Receiver<E>;

    /// A convenience method to receive either Inputs or Events inside a
    /// crossbeam select loop. Unlike send_input(), this one is used frequently
    /// because it doesn't require use of &self.
    fn recv_operation<T>(
        oper: crossbeam::channel::SelectedOperation,
        r: &Receiver<T>,
    ) -> Result<T, crossbeam::channel::RecvError> {
        let input_result = oper.recv(r);
        if let Err(e) = input_result {
            eprintln!(
                "ProvidesService: While attempting to receive from {:?}: {}",
                *r, e
            );
        }
        input_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CrossbeamChannel;
    use crossbeam::channel::Select;
    use std::time::Duration;

    #[derive(Debug)]
    enum TestServiceInput {
        Add(u8, u8),
    }

    #[derive(Debug, PartialEq)]
    enum TestServiceEvent {
        Added(u8),
    }

    #[derive(Debug)]
    struct TestService {
        inputs: CrossbeamChannel<TestServiceInput>,
        events: CrossbeamChannel<TestServiceEvent>,
    }
    impl Default for TestService {
        fn default() -> Self {
            let r = Self {
                inputs: Default::default(),
                events: Default::default(),
            };

            let receiver = r.inputs.receiver.clone();
            let sender = r.events.sender.clone();
            std::thread::spawn(move || {
                while let Ok(input) = receiver.recv() {
                    match input {
                        TestServiceInput::Add(a, b) => {
                            let _ = sender.send(TestServiceEvent::Added(a + b));
                        }
                    }
                }
            });

            r
        }
    }
    impl ProvidesService<TestServiceInput, TestServiceEvent> for TestService {
        fn sender(&self) -> &Sender<TestServiceInput> {
            &self.inputs.sender
        }

        fn receiver(&self) -> &Receiver<TestServiceEvent> {
            &self.events.receiver
        }
    }

    #[test]
    fn provides_service() {
        let s = TestService::default();
        let _ = s.send_input(TestServiceInput::Add(1, 2));

        let mut sel = Select::default();

        let test_receiver = s.receiver().clone();
        let test_index = sel.recv(&test_receiver);

        loop {
            match sel.select_timeout(Duration::from_secs(1)) {
                Ok(oper) => match oper.index() {
                    index if index == test_index => {
                        if let Ok(input) = TestService::recv_operation(oper, &test_receiver) {
                            match input {
                                TestServiceEvent::Added(sum) => {
                                    assert_eq!(sum, 3);
                                    break;
                                }
                            }
                        }
                    }
                    other => {
                        panic!("Unexpected select index: {other}");
                    }
                },
                Err(e) => {
                    panic!("select failed: {e:?}");
                }
            }
        }
    }
}
