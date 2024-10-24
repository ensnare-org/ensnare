# ensnare-services

This crate wraps certain crates used by the [ensnare
crate](https://crates.io/crates/ensnare) and makes them easier to use with
[crossbeam channels](https://crates.io/crates/crossbeam-channel).

Services available:

* **audio**: wraps the [cpal](https://crates.io/crates/cpal) audio-interface crate.
* **midi**: wraps the [midir](https://crates.io/crates/midir) MIDI-interface crate.

See the examples directory for usage, or visit
[docs.rs](https://docs.rs/ensnare-services/latest/ensnare-services/) to see the
API documentation.

## Crates in the Ensnare family

* [ensnare-proc-macros](https://crates.io/crates/ensnare-proc-macros): proc macros ([docs](https://docs.rs/ensnare-proc-macros/) [src](https://github.com/ensnare-org/ensnare/tree/main/crates/proc-macros))
* [ensnare-services](https://crates.io/crates/ensnare-services): service wrappers ([docs](https://docs.rs/ensnare-services/) [src](https://github.com/ensnare-org/ensnare/tree/main/crates/services))
* [ensnare-toys](https://crates.io/crates/ensnare-toys): simple instruments using ensnare infrastructure ([docs](https://docs.rs/ensnare-toys/) [src](https://github.com/ensnare-org/ensnare/tree/main/crates/toys))
* [ensnare](https://crates.io/crates/ensnare): digital audio creation ([docs](https://docs.rs/ensnare/) [src](https://github.com/ensnare-org/ensnare))
