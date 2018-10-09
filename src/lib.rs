// industrial-io/src/lib.rs
//
//!
//!

//#[macro_use]
extern crate nix;

#[macro_use]
extern crate error_chain;

extern crate libiio_sys as ffi;

use std::ffi::CStr;
use std::os::raw::c_char;

pub use context::*;
pub use device::*;
pub use channel::*;
pub use buffer::*;
pub use errors::*;

pub mod context;
pub mod device;
pub mod channel;
pub mod buffer;
pub mod errors;

fn cstring_opt(pstr: *const c_char) -> Option<String> {
    if pstr.is_null() {
        None
    }
    else {
        let name = unsafe { CStr::from_ptr(pstr) };
        Some(name.to_str().unwrap_or_default().to_string())
    }
}


// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
