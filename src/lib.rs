// industrial-io/src/lib.rs
//
// Copyright (c) 2018-2020, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//!
//! The Rust Industrial I/O crate for Linux.
//!
//! This is a Rust wrapper for _libiio_, a library for high-performance
//! analog I/O from Linux user-space. It interacts with Linux Industrial I/O
//! (IIO) devices such as A/D's, D/A's, accelerometers, pressure and
//! temperature sensors, magnetometers, and so on.
//!
//! For more information, see:
//!
//!   [IIO Wiki](https://wiki.analog.com/software/linux/docs/iio/iio)
//!
//!   [libiio Wiki](https://wiki.analog.com/resources/tools-software/linux-software/libiio)
//!

// Lints
// This may be overkill.
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_uint},
    slice, str, fmt,
};

use libiio_sys::{self as ffi};
use nix::errno;

pub use crate::buffer::*;
pub use crate::channel::*;
pub use crate::context::*;
pub use crate::device::*;
pub use crate::errors::*;

mod macros;

pub mod buffer;
pub mod channel;
pub mod context;
pub mod device;
pub mod errors;

/// According to the IIO samples, internal buffers need to be big enough
/// for attributes coming back from the kernel.
const ATTR_BUF_SIZE: usize = 16384;

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

pub(crate) fn sys_result<T>(ret: i32, result: T) -> Result<T> {
    if ret < 0 {
        Err(errno::from_i32(-ret).into())
    }
    else {
        Ok(result)
    }
}

// --------------------------------------------------------------------------

/// A struct to hold version numbers
#[derive(Debug, PartialEq)]
pub struct Version {
    /// The Major version number
    pub major: u32,
    /// The Minor version number
    pub minor: u32,
    /// The git tag for the release
    pub git_tag: String,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{} tag: {}", self.major, self.minor, self.git_tag)
    }
}

// --------------------------------------------------------------------------

/// Gets the library version as (Major, Minor, Git Tag)
pub fn library_version() -> Version {
    let mut major: c_uint = 0;
    let mut minor: c_uint = 0;

    const BUF_SZ: usize = 8;
    let mut buf = vec![' ' as c_char; BUF_SZ];
    let pbuf = buf.as_mut_ptr();

    unsafe { ffi::iio_library_get_version(&mut major, &mut minor, pbuf) };

    let sgit = unsafe {
        if buf.contains(&0) {
            CStr::from_ptr(pbuf).to_owned()
        }
        else {
            let slc = str::from_utf8(slice::from_raw_parts(pbuf as *mut u8, BUF_SZ)).unwrap();
            CString::new(slc).unwrap()
        }
    };
    Version {
        major: major as u32,
        minor: minor as u32,
        git_tag: sgit.to_string_lossy().into_owned(),
    }
}

// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Just make sure version gives a consistent result.
    #[test]
    fn version() {
        let v1 = library_version();
        let v2 = library_version();
        assert!(v1 == v2);
    }
}
