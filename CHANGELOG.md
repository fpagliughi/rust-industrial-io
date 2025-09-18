# Change Log
# for Rust Industrial I/O

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.6.2  (Unreleased)

- Added methods to access a buffer as a slice
- [#27](https://github.com/fpagliughi/rust-industrial-io/pull/27) Added method to get mutable slice of channel buffer


## [v0.6.1](https://github.com/fpagliughi/rust-industrial-io/compare/v0.6.0..v0.6.1) - 2025-09-13

- Updated the Github Actions CI to resolve the dependencies for the MSRV with cargo resolver v3.
- Renamed the 'utilities' feature to 'utils' and removed it from the default build
- Bumped MSRV to v1.75 to appease dependencies.
- Fixed clippy warnings from v1.89 about elided lifetimes
- Bumped dependencies
    - clap v4.4
    - thiserror v2.0
- [PR #32](https://github.com/fpagliughi/rust-industrial-io/pull/32) Typo in `ChannelType::Light`
- [PR #33](https://github.com/fpagliughi/rust-industrial-io/pull/33) Fix build script platform behavior


## [v0.6.0](https://github.com/fpagliughi/rust-industrial-io/compare/v0.5.2..v0.6.0) - 2024-12-10

- Upgraded to Rust Edition 2021, MSRV 1.73.0
- New bindings in the -sys crate for _libiio_ v0.24 & v0.25
    - Cargo build features for selecting bindings to older libiio versions (v0.24, v0.23, etc)
    - Conditional features based on the version of _libiio_.
- Updated examples and utils to use `clap` v3.2, with forward-looking implementation.
- Added _buildtst.sh_ script for local CI testing. This runs the cargo _check, test, clippy,_ and _doc_ for the latest stable compiler and the MSRV.
- Fixed new clippy warnings.
- Updated `nix` dependency to v0.29
- Renamed `iio_info_rs` to `riio_info` to be compatible with naming of other utilities and examples.
- Converted to explicit re-exports to avoid ambiguous warnings.
- Added a mutable iterator for channel data in a buffer (to fill the buffer)
- Added lifetime to buffer iterator so as not to outlive the buffer.
- [Breaking]: Buffer iterator now returns a reference to the item in the buffer, to be consistent with mutable iterator and slice iterators.
- [PR #28](https://github.com/fpagliughi/rust-industrial-io/pull/28)-  Move set_num_kernel_buffers() to Device
- [PR #22](https://github.com/fpagliughi/rust-industrial-io/pull/22)-  Disable chrono default features to mitigate segfault potential in time crate
- Added initial CI support to test building and format. (Still can't run unit tests in CI due to iio kernel module requirements).


## [v0.5.2](https://github.com/fpagliughi/rust-industrial-io/compare/v0.5.1..v0.5.2) - 2023-02-03

- [PR #26](https://github.com/fpagliughi/rust-industrial-io/pull/26) - Added 'utilities' feature to be able to turn off build of binary applications (i.e. only build the library).
- [#21](https://github.com/fpagliughi/rust-industrial-io/issues/21) - Update nix dependency to avoid linking vulnerable version
- Updated dependencies for `clap` and `ctrlc` crates.


##  [v0.5.1](https://github.com/fpagliughi/rust-industrial-io/compare/v0.5.0..v0.5.1) - 2022-02-05

- `iio_info_rs` utility now supports network and URI contexts.
- [PR #19](https://github.com/fpagliughi/rust-industrial-io/pull/19) macOS build makes a distinction for Intel and non-Intel builds when searching for Homebrew Frameworks (libiio library).
- [PR #20](https://github.com/fpagliughi/rust-industrial-io/pull/20) Fix some clippy suggestions. Particularly cleaner casting of raw pointers, etc.


##  [v0.5.0](https://github.com/fpagliughi/rust-industrial-io/compare/v0.4.0..v0.5.0) - 2022-01-30

- Started loosening thread safety restrictions:
    - The `Context` is now `Send` and `Sync`. Internally it has canverted to using an `Arc` instead of an `Rc` to track it's internal data.
    - The `Device` is now `Send`.
    - For high performance with multiple device, though, it's still recommended to used fully-cloned contexts for each device
    - For now, `Channel` and `Buffer` objects are still `!Send` and `!Sync`. So they should live in the same thread as their channel.
- New functions to manipulate `Context` and `InnerContext` objects:
     - `Context::try_release_inner()` attempts to get the inner context out of the context wrapper.
     - `Context::try_deep_clone()` to make a new context around a deep copy of the inner context (and thus a copy of the C lib context).
     - `From<InnerContext> for Context`


##  [v0.4.0](https://github.com/fpagliughi/rust-industrial-io/compare/v0.3..v0.4.0) - 2022-01-28

- [#12](https://github.com/fpagliughi/rust-industrial-io/pull/12) Context construction now takes a `Backend` enumeration type.
- The `InnerContext` is now public and can be cloned and sent to another thread to create a cloned context in the other thread.
- [#15](https://github.com/fpagliughi/rust-industrial-io/issues/15) Generic `attr_read()` and `attr_write()` functions for devices, channels, and buffers.
- [#17](https://github.com/fpagliughi/rust-industrial-io/pull/17) macOS support (for network clients)
- Buffer attribute read/write functions and iterators moved into the `Buffer` struct.
- `Buffer` struct now contains a clone of the `Device` from which it was created.
- `Device` and `Channel` now support `Clone` trait.
- Updates to the examples for more/different hardware.
- New `Version` struct which is returned by the library and `Context` version query functions.


##  [v0.3](https://github.com/fpagliughi/rust-industrial-io/compare/v0.2..v0.3) - 2021-05-26

- Support for _libiio_ v0.21
- Updated error handling:
    - Support for `std::error`
    - Implementation changed to use `thiserror` (from *error_chain*)
    - Specific types defined for common errors instead of just string descriptions (`WrongDataType`, `BadReturnSize`, `InvalidIndex,` etc)
- New device capabilities:
    - _remove_trigger()_
    - _is_buffer_capable()_
- New utility app: _riio_stop_all_


## [v0.2](https://github.com/fpagliughi/rust-industrial-io/compare/v0.1..v0.2) - 2019-12-29

- Support for libiio v0.18
- Further implementation of _libiio_ functions for contexts, devices, channels, etc.
- Functions to read and write buffers, with and without conversions, and to convert individual samples to and from hardware format.
- [Breaking] Removed previous `ChannelType` for Input/Output as it conflicted with the library's channel types of `Voltage`, `Current`, `Power`, etc, and implemented the library type.
- Contexts have a ref-counted "inner" representation using _Rc<>_, and can be "cloned" quickly by incrementing the count. (Thanks, @skrap!)
 - Devices carry a cloned reference to the context that created them, thus keeping the context alive until the last device using it gets dropped.
 - Some clippy-recommended lints.
 - Example app to collect and process data a buffer at a time, with conversions.

## v0.1 - 2018-11-21

- Initial version
- Basic functionality for contexts, devices, channels, and buffers.