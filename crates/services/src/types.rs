// Copyright (c) 2024 Mike Tsao

//! Data types shared among services.

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
