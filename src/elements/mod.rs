// Copyright (c) 2024 Mike Tsao

//! Building blocks for other parts of the system, especially musical
//! instruments and effects.

/// The most commonly used imports.
pub mod prelude {
    pub use super::transport::{Transport, TransportBuilder};
}

pub use transport::{Transport, TransportBuilder};

mod transport;
