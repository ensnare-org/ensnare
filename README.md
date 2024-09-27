# Ensnare: Generate digital audio in Rust

Ensnare is a Rust library for generating digital audio. It is pre-release. Its
API is not yet stable.

## Usage

See the examples directory for usage, or visit
[docs.rs](https://docs.rs/ensnare/latest/ensnare/) to see the API documentation.

## Development

### First-time setup

To set up your Linux machine for development, see the `apt install` packages in
[`.github/workflows/build.yml`](./.github/workflows/build.yml). Other platforms
don't have special dependencies.

### Useful Cargo commands

* `deb`
* `expand`
* `fmt`
* `machete`
* `release`
* `tree`

### Bash commands that I use during Ensnare development

* `./precheck`: runs formatting and tests that should precede any commit.
* `./do-release`: Generates a new release and pushes it to
  [GitHub](https://github.com/ensnare-org/ensnare) and
  [crates.io](https://crates.io/crates/ensnare).

### How to release to crates.io

```bash
# start at the root of the project
cd ~/src/ensnare

# run our checks and make sure everything is clean and tidy
./precheck 

# release ensnare-proc-macros if necessary
#
pushd crates/proc-macros
# dry run; make sure everything works and that a new version is needed
cargo release patch
# actually do it
cargo release patch --sign -x
popd

# release main ensnare crate if necessary
#
# dry run; make sure everything works and that a new version is needed
cargo release alpha
# actually do it
cargo release alpha --sign -x

# release ensnare-services if necessary
#
pushd crates/services
# dry run; make sure everything works and that a new version is needed
cargo release patch
# actually do it
cargo release patch --sign -x
popd
```

## Crates in the Ensnare family

* [ensnare-proc-macros](https://crates.io/crates/ensnare-proc-macros): proc macros ([docs](https://docs.rs/ensnare-proc-macros/) [src](https://github.com/ensnare-org/ensnare/tree/main/crates/proc-macros))
* [ensnare-services](https://crates.io/crates/ensnare-services): service wrappers ([docs](https://docs.rs/ensnare-services/) [src](https://github.com/ensnare-org/ensnare/tree/main/crates/services))
* [ensnare](https://crates.io/crates/ensnare): digital audio creation ([docs](https://docs.rs/ensnare/) [src](https://github.com/ensnare-org/ensnare))
