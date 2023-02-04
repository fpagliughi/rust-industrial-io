// libiio-sys/src/lib.rs
//
//! Wrapper for the Linux Industrial I/O user-space library, _libiio_.
//!
//! Build Features can be used to select bindings for one of several versions
//! that might be installed in the system.
//!
//! #### Default Features
//!
//! * **libiio_v0_24** Bindings for libiio v0.24
//!
//! #### Optional Feature
//!
//! Select only one feature to specify a version for libiio:
//!
//! * **libiio_v0_24** Bindings for libiio v0.24
//! * **libiio_v0_23** Bindings for libiio v0.23
//! * **libiio_v0_21** Bindings for libiio v0.21
//! * **libiio_v0_19** Bindings for libiio v0.19
//!

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(rustdoc::broken_intra_doc_links)]

// Temporary
#![allow(dead_code)]

// Bindgen uses u128 on some rare parameters
#![allow(improper_ctypes)]

// ----- Use bindings for libiio v0.24 -----

#[cfg(all(unix, feature = "libiio_v0_24", target_pointer_width = "64"))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/bindings-0.24_64.rs"));

#[cfg(all(unix, feature = "libiio_v0_24", target_pointer_width = "32"))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/bindings-0.24_32.rs"));

// ----- Use bindings for libiio v0.23 -----

#[cfg(all(unix, feature = "libiio_v0_23", target_pointer_width = "64"))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/bindings-0.23_64.rs"));

#[cfg(all(unix, feature = "libiio_v0_23", target_pointer_width = "32"))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/bindings-0.23_32.rs"));

// ----- Use bindings for libiio v0.21 -----

#[cfg(all(unix, feature = "libiio_v0_21", target_pointer_width = "64"))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/bindings-0.21_64.rs"));

#[cfg(all(unix, feature = "libiio_v0_21", target_pointer_width = "32"))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/bindings-0.21_32.rs"));

// ----- Use bindings for libiio v0.19 -----

#[cfg(all(unix, feature = "libiio_v0_19", target_pointer_width = "64"))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/bindings-0.19_64.rs"));

#[cfg(all(unix, feature = "libiio_v0_19", target_pointer_width = "32"))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/bindings-0.19_32.rs"));


