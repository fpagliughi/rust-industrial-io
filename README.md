# Rust Industrial I/O for Linux

![Crates.io](https://img.shields.io/crates/d/industrial-io)

Rust library crate for using the Linux Industrial I/O (IIO) subsytem, primarily used for the input and output of analog data from a Linux system in user space. See the [IIO Wiki](https://wiki.analog.com/software/linux/docs/iio/iio).

The current version is a wrapper around the user-space C library, [libiio](https://github.com/analogdevicesinc/libiio).  Subsequent versions may access the interface the kernel ABI directly.

To use in an application, add this to _Cargo.toml:_

```toml
[dependencies]
industrial-io = "0.3"
```

## Pre-release Note

This is a pre-release verion of the crate. The API is stabilizing, but is still under active development and may change before a final release.

This initial development work wrappers a _specific_ version (v0.21) of _libiio_. It assumes that the library is pre-installed on the build system and the target system.

## Latest News

An effort is underway to get this crate to production quality.  It includes:

- Full coverage of the _libiio_ API - or as much as makes sense.
- A complete set of working examples.
- Unit tests and CI

To keep up with the latest announcements for this project, follow:

**Twitter:**  [@fmpagliughi](https://twitter.com/fmpagliughi)

### Unreleased Features in This Branch

- [#12](https://github.com/fpagliughi/rust-industrial-io/pull/12) Context construction now takes a `Backend` enumeration type. (Thanks @Funky185540)
- The `InnerContext` is now public and can be cloned and sent to another thread to create a cloned context in the other thread.
- Updates to the examples for more/different hardware.

### New in v0.3

- Support for _libiio_ v0.21
- Updated error handling:
    - Support for `std::error`
    - Implementation changed to use `thiserror` (from *error_chain*)
    - Specific types defined for common errors intead of just string descriptions (`WrongDataType`, `BadReturnSize`, `InvalidIndex,` etc)
- New device capabilities:
    - _remove_trigger()_
    - _is_buffer_capable()_
- New utility app: _riio_stop_all_

## The Basics

The C _libiio_ library provides a user-space interface to the Linux Industrial I/O subsystem for interfacing to (possibly high-speed) analog hardware, such as A/D's, D/A's, accelerometers, gyroscopes, etc. This crate provides a fairly thin, safe, wrapper around the C library.

To access any physical devices you must first create a `Context` which can use one of several back-ends, the most common of which are:

- "local" - To access hardware on the local machine
- "network" - To access hardware on a remote machine that is running the IIO network daemon, `iiod`

There are also backends for USB and serial devices.

The default context will use the local machine, but can be overridden with the `IIOD_REMOTE` environment variable which, when set, gives the host name for a network context.

A given context, once created, can then be queried for information about the devices it contains and the channels provided by each device. The context, devices and their channels contain _attributes_ that read and set data and the parameters for collecting it.

Data sampling and/or output can be started by _trigger_ devices which can be hardware or software timers to collect periodic data, or triggers based on external events like GPIO inputs. Note, too, that some devices can self-trigger.

The library can also use the in-kernel ring buffers to collect data at higher speeds with less jitter.

There are a number of applications in the [examples/](https://github.com/fpagliughi/rust-industrial-io/tree/master/examples) directory.

### Hardware and Driver Peculiarities

The Linux IIO subsystem and _libiio_ abstract a large number of different types of hardware with considerably different feature sets. Between the different capabilities of the hardware and the drivers written for them, applications can often see weird and unexpected results when starting out with a new device. The example applications are not guaranteed to work out-of-the box with all different types of hardware. But they provide a decent template for the most common usage. Some modifications and experimentation are often required when working with new devices.

### Thread Safety

The contexts and devices in the underlying _libiio_ are not thread safe. Therefore, neither are the wrapped versions in this Rust library. But the Rust library will enforce the thread safety requirements. For the most part, objects from this library are neither `Send` nor `Sync`. A `Context` object can only be used in a single thread, and devices, channels, and buffers can only be used in the same thread as the context that created them.

The _physical_ devices described by an IIO context can sometimes be manipulated by different threads. This is highly hardware dependent, but when allowed, there way to so it is to use a separate `Context` instance for each thread. There are two ways to do this:

1. Simply create a new `Context` object in each thread using the same URI, etc.
2. Create a context and then make clones of its `InnerContext` object and send those to other threads. Each one can then be used to create a new `Context` instance in that thread.

This second option can be done like this:

    let ctx = Context::new()?;
    let cti = ctx.try_clone_inner()?;

    let thr = thread::spawn(move || {
        let thr_ctx = Context::from_inner(cti);
        // ...
    });

Here the inner context is cloned to the `cti` object which is moved into the thread and consumed to create a new context object, `thr_ctx`.

An alternate way to share devices across threads and processes is to run the IIO network daemon on the local machine and allow it to control the local context. Then multiple client applications can access it from _localhost_ using a network context. The daemon will serialize access to the device and let multiple clients share it. Each thread in the client would still need a separate network context.

## Testing the Crate

A great thing about the user-space IIO libraries is that, if you're developing on a fairly recent Linux host, you can start experimenting without having to do development on a board. You can run the IIO server daemon on an embedded board, and then use this crate to communicate with it over a network connection. When your application is working, you can then compile it for the target board, test it natively, and deploy.

Alternately, you can test with a mock, "dummy" context on a development host. This is a kernel module that simulates several devices. It can be used from a local context on a host machine to do some initial development and test of the library. See below for details on loading it into the kernel.

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

Install _libiio_ v0.21 on the target board. If you're developing on a Linux host, install the same version of the library there so that you can do some development on the host,

### Check the version on the target

If you're using a BeagleBone Black, an old library may have shipped with the distro. Install the latest distribution for the board. (This was tested with _Debian 9.5 2018-10-07 4GB SD IoT_).

Log onto the board and check the version:

```
$ iiod --version
0.21
```

If this is less than 0.21, remove the Debian packages and install from sources.

First, get rid of the existing library and utilities:

```
$ sudo apt-get purge libiio-utils libiio0
```

### Build from sources

Install the pre-requisites for the build:

```
$ sudo apt-get install cmake flex bison libxml2-dev libserialport-dev
```

And then download the library sources and build:

```
$ cd /tmp
$ wget https://github.com/analogdevicesinc/libiio/archive/v0.21.tar.gz
$ tar -xf v0.21.tar.gz 
$ cd libiio-0.21/
$ mkdir build ; cd build
$ cmake .. && make && sudo make install
```

Then check that the version installed properly:

```
$ iiod --version
0.21
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
