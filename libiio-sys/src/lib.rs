// libiio-sys/src/lib.rs
//
//!
//!

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Temporary
#![allow(dead_code)]

// Bring in the bindgen bindings of "iio.h"
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/bindings-0.15.rs"));

// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
}
