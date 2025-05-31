// libiio-sys/src/lib.rs
//
//! Wrapper for the Linux Industrial I/O user-space library, _libiio_.
//!
//! Versions 0.19 through 0.25 are supported and will have bindings
//! automatically generated.
//!

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(rustdoc::broken_intra_doc_links)]
// Temporary
#![allow(dead_code)]
// Bindgen uses u128 on some rare parameters
#![allow(improper_ctypes)]

include!(concat!(
    env!("OUT_DIR"),
    "/iio_bindings.rs"
));