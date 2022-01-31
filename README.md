# Rust Industrial I/O for Linux

![Crates.io](https://img.shields.io/crates/d/industrial-io)

Rust library crate for using the Linux Industrial I/O (IIO) subsystem, primarily used for the input and output of analog data from a Linux system in user space. See the [IIO Wiki](https://wiki.analog.com/software/linux/docs/iio/iio).

The current version is a wrapper around the user-space C library, [libiio](https://github.com/analogdevicesinc/libiio).  Subsequent versions may access the interface the kernel ABI directly.

To use in an application, add this to _Cargo.toml:_

```toml
[dependencies]
industrial-io = "0.5"
```

## Pre-release Note

This is a pre-release version of the crate. The API is stabilizing, but is still under active development and may change before a final release.

This initial development work wrappers a _specific_ version (v0.21) of _libiio_. It assumes that the library is pre-installed on the build system and the target system.

## Contributing

Contributions to this project are gladly welcomed. Just keep a few things in mind:

- Please make all Pull Requests against the `develop` branch of the repository. We prefer to keep the master branch relatively stable between releases and to do integration and testing in the develop branch.
- Please keep individual Pull Requests to a single topic.
- Please do not reformat code with other updates. Any code reformatting should be in a separate commit or PR. The formatting specification is in `.rustfmt.toml` and currently requires the _nightly_ release.

Contributions are particularly welcome for any adjustments or feedback pertaining to different IIO device. If you test, work, or have any trouble with specific IIO hardware or drivers, let us know. 

New examples for different hardware are also requested.

## Latest News

Overall, an effort is underway to get this crate to production quality.  It includes:

- Full coverage of the _libiio_ API - or as much as makes sense.
- A complete set of working examples.
- Unit tests and CI

To keep up with the latest announcements for this project, follow:

**Twitter:**  [@fmpagliughi](https://twitter.com/fmpagliughi)

### New in Version 0.5.0

- Started loosening thread safety restrictions:
    - The `Context` is now `Send` and `Sync`. Internally it has converted to using an `Arc` instead of an `Rc` to track it's internal data.
    - The `Device` is now `Send`.
    - For high performance with multiple device, though, it's still recommended to used fully-cloned contexts for each device
    - For now, `Channel` and `Buffer` objects are still `!Send` and `!Sync`. So they should live in the same thread as their channel.
- New functions to manipulate `Context` and `InnerContext` objects:
     - `Context::try_release_inner()` attempts to get the inner context out of the context wrapper.
     - `Context::try_deep_clone()` to make a new context around a deep copy of the inner context (and thus a copy of the C lib context).
     - `From<InnerContext> for Context`

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

## Implementation Details

The Rust Industrial I/O library is a fairly thin wrapper around the C _libiio_, with some features thrown in to give it a more Rust-y feel.

### Library Wrapper

To do anything with _libiio_, the application must first create a `Context`to either manipulate the hardware on the local device (i.e. a _local_ context), or to communicate with hardware on a remote device such as over a network connection. Creating a local context is a fairly heavyweight operation compared to other library operations in that it will scan the hardware and build up a local representation in memory.

The context is thus a snapshot of the hardware at the time at which it was created. Any hardware that is added outside of the context - such as another process creating a new _hrtimer_, will not be reflected in it. A new context would need to be created to re-scan the hardware.

But then, finding hardware is very efficient in that it just searches through the data structures in the context. A call like `ctx.find_device('adc0')` just looks for a string match in the list of hardware devices, and the pointer returned by the underlying library call is juts a reference to an existing context data structure.

Nothing is created or destroyed when new Rust hardware structures are declared, such as `Device` or `Channel`. Therefore the Rust structures can easily be cloned by copying the pointer to the library structure.

The Rust `Context` object is just a reference-counted smart pointer to an `InnerContext`. This makes it easy to share the C context between different objects (devices, channels, etc), and to manage its lifetime. The `InnerContext` actually wraps the C context. When it goes out of scope, typically when the last reference disappears, the C context is destroyed. Cloning the `InnerContext` creates a full copy of the C library's context.

This creates some confusion around "cloning" a Context. Since a `Context` is just a thread-safe, reference counted smart pointer to that inner context, cloning it just creates a new, shared pointer to the existing inner/C context. This makes it easy to share the context and guarantee it's lifetime between multiple `Device` objects created from it. Cloning a `Context` and sending the clone to another thread will actually then _share_ the `InnerContext` (and thus C context) between the two threads.
 
Often, however, when using separate threads to manage each device, it can be more efficient to create a fully separate C context for each thread. To do this, a "deep" clone of the `Context` is necessary. This is simply a clone of the `InnerContext`, and creating new smart pointers around that inner context. So it is a clone of the `InnerContext` which actually makes a copy of the C library context. See the next section for details.
 
### Thread Safety

Early versions of this library (v0.4.x and before) were written with the mistaken belief that the underling _libiio_ was not thread-safe. Some public information about the library was a little misleading, but with input from a maintainers of the library and additional published information, thread restrictions are slowly being lifted from this library.

Starting in v0.5, the following is now possible:

- `InnerContext`, which actually wraps the C library context, is now `Sync` in addition to being `Send`. It can be shared between threads.
- `Context` is now implemented with an `Arc` to point to its `InnerContext`. So these references to the inner context can be sent to different threads and those threads can share the same context.
- The `Device` objects, which hold a `Context` reference, are now `Send`. They can be moved to a different thread than the one that created the context.
- For now, the `Channel` and `Buffer` objects are still `!Send` and `!Sync`, and need to live in the same thread with the `Device`, but these restrictions may be loosened as we figure out which specific operations are not thread safe.
- The `Buffer::refill()` function now take a mutable reference to self, `&mut self`, in preparation of loosening thread restrictions on the buffer. The buffer definitely can not be filled by two different threads at the same time.

Even with these new thread capabilities, when the _physical_ devices described by an IIO context can be manipulated by different threads, it is often still desirable to use a separate `Context` instance for each thread. There are two ways to do this:

1. Simply create a new `Context` object in each thread using the same URI, etc. These might not be exactly the same if some new hardware was added or removed between the time any two contexts were created. But this is perhaps rare.
2. Create a context and then make clones of its `InnerContext` object and send those to other threads. Each one can then be used to create a new `Context` instance in that thread.

This second option is considerably more efficient and can be done in several ways.

One way is to do a "deep" clone of a context and send it to the other thread:

    let ctx = Context::new()?;
    let thr_ctx = ctx.try_deep_clone()?;
    
    thread::spawn(move || {
        let dev = thr_ctx.find_device("somedevice")?
        // ...
    });

This makes a copy of the inner context which clones the C library context. It then sends the one-and-only reference to the other thread, giving it exclusive access to that C context.

Alternately, to be explicit about cloning the inner context, this can be done:

    let ctx = Context::new()?;
    let cti = ctx.try_clone_inner()?;

    thread::spawn(move || {
        let thr_ctx = Context::from(cti);
        let dev = thr_ctx.find_device("somedevice")?
        // ...
    });

Here the inner context is cloned to the `cti` object which is moved into the thread and consumed to create a new context object, `thr_ctx`. This procedure was required in the earlier versions of library, prior to v0.5.0.

An alternate way to share devices across threads and processes is to run the IIO network daemon on the local machine and allow it to control the local context. Then multiple client applications can access it from _localhost_ using a network context. The daemon will serialize access to the device and let multiple clients share it. Each thread in the client would still need a separate network context.

The thing to keep in mind is that although Rust can enforce thread safety within a single process, the overall IIO subsystem is exposed to the other processes. Different devices and drivers might expose access differently. It is up to the system designer to insure that processes using IIO don't interfere with each other.

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

### macOS

The crate is also compatible with macOS, though only the network contexts are available. The libiio framework can be built from source or installed from a community homebrew formula:

```
brew install tfcollins/homebrew-formulae/libiio
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

This is a fairly standard Rust wrapper project around a C library. It contains an unsafe _"-sys"_ sub-crate to wrap the C library API, and a higher-level, safe, Rust library in the main crate. To build them:

```
$ cargo build
```

There are also a number of example applications. They can all be built with:

```
$ cargo build --examples
```
