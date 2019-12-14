// industrial-io/src/lib.rs
//
// Copyright (c) 2018, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
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

use nix::errno;
use nix::Error::Sys as SysError;

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

// --------------------------------------------------------------------------

/// Gets an optional string value from a C const char pointer.
/// If the pointer is NULL, this returns `None` otherwise it converts the
/// string and returns it.
fn cstring_opt(pstr: *const c_char) -> Option<String> {
    if pstr.is_null() {
        None
    }
    else {
        let name = unsafe { CStr::from_ptr(pstr) };
        Some(name.to_str().unwrap_or_default().to_string())
    }
}

pub fn sys_result<T>(ret: i32, result: T) -> Result<T> {
    if ret < 0 {
        bail!(SysError(errno::from_i32(ret)))
    }
    Ok(result)
}

// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
