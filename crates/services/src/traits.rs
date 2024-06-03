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
