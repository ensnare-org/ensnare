// Copyright (c) 2024 Mike Tsao

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! Ensnare helps create digital audio.

pub use version::app_version;

/// A collection of imports that are useful to users of this crate. `use
/// ensnare::prelude::*;` for easier onboarding.
pub mod prelude {
    // TODO
}

mod version;

/// Adds two integers.
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
