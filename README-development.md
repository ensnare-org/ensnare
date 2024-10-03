# Ensnare development

This document is for development *of* Ensnare (adding to or changing this
repository). If you're interested in development *with* it (using the Ensnare
crate in your app or crate), See the [main README](./README.md).

## First-time setup

To set up your Linux machine for development, see the `apt install` packages in
[`.github/workflows/build.yml`](./.github/workflows/build.yml). Other platforms
don't have special dependencies.

## Useful Cargo commands

* `deb`
* `expand`
* `fmt`
* `machete`
* `release`
* `tree`

## Bash commands that I use during Ensnare development

* `./precheck`: runs formatting and tests that should precede any commit.
* `./do-release`: Generates a new release and pushes it to
  [GitHub](https://github.com/ensnare-org/ensnare) and
  [crates.io](https://crates.io/crates/ensnare).

## How to release to crates.io

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
