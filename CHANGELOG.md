# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

###  [v0.3](https://github.com/fpagliughi/rust-industrial-io/compare/v0.2..v0.3) - 2021-05-26


- Support for _libiio_ v0.21
- Updated error handling:
    - Support for `std::error`
    - Implementation changed to use `thiserror` (from *error_chain*)
    - Specific types defined for common errors intead of just string descriptions (`WrongDataType`, `BadReturnSize`, `InvalidIndex,` etc)
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