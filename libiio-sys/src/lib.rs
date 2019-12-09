// libiio-sys/src/lib.rs
//
//!
//!

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Temporary
#![allow(dead_code)]

// Bindgen uses u128 on some rare parameters
#![allow(improper_ctypes)]
// Bring in the bindgen bindings of "iio.h"
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/bindings-0.18.rs"));

// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
}
