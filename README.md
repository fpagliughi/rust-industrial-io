# Rust Industrial I/O for Linux

This is a Rust library crate for using the Linux Industrial I/O subsytem.

See the [IIO Wiki](https://wiki.analog.com/software/linux/docs/iio/iio).

The initial, 1.0, version will be a wrapper around [libiio](https://github.com/analogdevicesinc/libiio).  Subsequent versions may access the interface directly.

## Pre-release notes

This is an early, pre-1.0 release verion of the crate. The API is under active development and may change repeatedly.

The initial version is a wrapper around a specific version, v0.15, of _libiio_. It assumes that the library is installed on the target system.