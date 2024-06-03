// Copyright (c) 2024 Mike Tsao

use crossbeam::channel::{Receiver, Sender};

/// A convenience struct to bundle both halves of a crossbeam channel together.
#[derive(Debug)]
pub struct CrossbeamChannel<T> {
    #[allow(missing_docs)]
    pub sender: Sender<T>,
    #[allow(missing_docs)]
    pub receiver: Receiver<T>,
}
impl<T> Default for CrossbeamChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = crossbeam::channel::unbounded();
        Self { sender, receiver }
    }
}

/// Same idea but only for bounded of bounds 1.
#[derive(Debug)]
pub struct BoundedCrossbeamChannel<T> {
    #[allow(missing_docs)]
    pub sender: Sender<T>,
    #[allow(missing_docs)]
    pub receiver: Receiver<T>,
}
impl<T> Default for BoundedCrossbeamChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = crossbeam::channel::bounded(1);
        Self { sender, receiver }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crossbeam_channel() {
        let channel = CrossbeamChannel::default();

        let _ = channel.sender.send(42);

        assert_eq!(channel.receiver.recv().unwrap(), 42);
    }
}
