# Ensnare: Generate digital audio in Rust

Ensnare is a Rust library for generating digital audio. It is pre-release. Its
API is not yet stable.

## Usage

See the examples directory for usage, or visit
[docs.rs](https://docs.rs/ensnare/latest/ensnare/) to see the API documentation.

## Development

To set up your Linux machine for development, see the `apt install` packages in
[`.github/workflows/build.yml`](./.github/workflows/build.yml). Other platforms
don't have special dependencies.

Cargo commands that I like:

* `deb`
* `expand`
* `fmt`
* `machete`
* `release`
* `tree`

Various Bash commands that I use during Ensnare development:

* `./precheck`: runs formatting and tests that should precede any commit.
* `cargo release --workspace -x alpha`: Generates a new release and pushes it to
  [GitHub](https://github.com/ensnare-org/ensnare) and
  [crates.io](https://crates.io/crates/ensnare).
