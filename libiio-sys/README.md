# Rust wrapper for Linux Industrial I/O library, _libiio_

![Crates.io](https://img.shields.io/crates/d/libiio-sys)

Rust wrapper for the Linux Industrial I/O user-space library, `libiio`. This provides high-performance analog input and output on Linux systems.

Currently this defaults to bindings for libiio v0.25.

To use in an application, add this to _Cargo.toml:_

```toml
[dependencies]
libiio-sys = "0.4"
```

## Generating Bindings

Bindings for different versions of the C library can be generated using the [bindgen command-line tools](https://rust-lang.github.io/rust-bindgen/command-line-usage.html).

Run `bindgen` over the `iio.h` header from the desired version of the library, outputting the results to a file,

```
bindings-<version>_<size>.rs
```

where `<version>` is the _libiio_ version and `<size>` is the target CPU word size (typically 32 or 64).

So, for example, this is how we generated them v0.25, on a 64-bit system:

First we cloned the repo and checked out the proper version:

```
$ git clone https://github.com/analogdevicesinc/libiio.git
$ cd libiio
$ git checkout v0.25
```

Then into the directory for this repo, we ran bindgen on the header saving the result in the `bindings` directory:

```
$ cd industrial-io/libiio-sys/
$ bindgen ~/libiio/iio.h -o bindings/bindings-0.25_64.rs
```