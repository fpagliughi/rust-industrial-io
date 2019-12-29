# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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