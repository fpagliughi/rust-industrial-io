# Rust Industrial I/O for Linux

![Crates.io](https://img.shields.io/crates/d/industrial-io)


This is a Rust library crate for using the Linux Industrial I/O (IIO) subsytem, primarily used for the input and output of analog data from a Linux system. See the [IIO Wiki](https://wiki.analog.com/software/linux/docs/iio/iio).

The current version is a wrapper around the user-space C library, [libiio](https://github.com/analogdevicesinc/libiio).  Subsequent versions may access the interface the kernel ABI directly.

## Pre-release notes

This is a pre-release verion of the crate. The API is stabilizing, but is still under active development and may change before a final release.

This initial development work wrappers a _specific_ version (v0.18) of _libiio_. It assumes that the library is installed on the target system.

## Latest News

An effort is underway to get this crate to production quality. This will hopefully be complete by early 2020. It includes:

- Full coverage of the _libiio_ API - or as much as makes sense.
- A complete set of working examples.
- Unit tests and CI

### Unreleased features in this branch:

- Support for libiio v0.18
- Further implementation of _libiio_ functions for contexts, devices, channels, etc.
- Functions to read and write buffers, with and without conversions, and to convert individual samples to and from hardware format.
- [Breaking] Removed previous `ChannelType` for Input/Output as it conflicted with the library's channel types of `Voltage`, `Current`, `Power`, etc, and implemented the library type.
- Contexts have a ref-counted "inner" representation using _Rc<>_, and can be "cloned" quickly by incrementing the count. (Thanks, @skrap!)
 - Devices carry a cloned reference to the context that created them, thus keeping the context alive until the last device using it gets dropped.
 - Some clippy-recommended lints.
 - Sample to collect and process data a buffer at a time.

## Testing the Crate

A great thing about the user-space IIO libraries is that, if you're developing on a fairly recent Linux host, you can start experimenting without having to do development on a board. You can run the IIO server daemon on an embedded board, and then use this crate to communicate with it over a network connection. When your application is working, you can then compile it for the target board and test it natively. Alternately, you can test with a mock, "dummy" context on a development host.

### BeagleBone

Several maker boards can be used to try out the Industrial I/O subsystem pretty easily. The BeagleBone Black and Green have the on-board AM335X A/D, and the BeagleBone AI has an STM touchscreen chip that can be used for analog input.

 The IIO library for the BeagleBones support individual and buffered sampling, though without external trigger support.  The recent [Debian 9.x IoT](https://beagleboard.org/latest-images) distributions for the board have IIO compiled into the kernel which can be used out of the box - although the user-space library, _libiio_, should be upgraded (see below).
 
### Linux Development Host

Several modern Linux distributions, such as Ubuntu 18.04, have IIO modules compiled for the kernel, such as the _dummy_ context. These can be loaded into the kernel, like:
```
$ sudo modprobe iio_dummy
$ sudo modprobe iio_trig_hrtimer
```
Once loaded, the _configfs_ can be used to create devices and triggers. The _load_dummy.sh_ script included in this repository can be used to load the modules and configure a device, suitable for basic experiments or running the unit tests.
```
$ sudo ./load_dummy.sh
```

## Installing the C Library

Install _libiio_ v0.18 on the target board. If you're developing on a Linux host, install the same version of the library there so that you can do some development on the host,

### Check the version on the target

If you're using a BeagleBone Black, an old library may have shipped with the distro. Install the latest distribution for the board. (This was tested with _Debian 9.5 2018-10-07 4GB SD IoT_).

Log onto the board and check the version:

```
$ iiod --version
0.18
```

If this is less than 0.18, remove the Debian packages and install from sources.

First, get rid of the existing library and utilities:

```
$ sudo apt-get purge libiio-utils libiio0
```

### Build from sources

Then install the pre-requisites for the build:

```
$ sudo apt-get install cmake flex bison libxml2-dev
```

And then download the library sources and build:

```
$ cd /tmp
$ wget https://github.com/analogdevicesinc/libiio/archive/v0.18.tar.gz
$ tar -xf v0.18.tar.gz 
$ cd libiio-0.18/
$ mkdir build ; cd build
$ cmake .. && make && sudo make install
```

Then check that the version installed properly:

```
$ iiod --version
0.18
```

## Build the Rust Crate

This is a fairly standard Rust wrapper project around a C library. It contains an unfafe _"-sys"_ sub-crate to wrap the C library API, and a higher-level, safe, Rust library in the main crate. To build them:

```
$ cargo build
```

There are also a number of example applications. They can all be built with:

```
$ cargo build --examples
```
