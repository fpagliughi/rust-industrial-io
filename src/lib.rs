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
// This may be overkill, but it's keeping me honest.
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    fmt,
    os::raw::{c_char, c_int, c_uint, c_void},
    slice, str,
    str::FromStr,
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

/// Trait to convert a value to a proper attribute string.
pub trait ToAttribute: fmt::Display + Sized {
    /// Converts the attribute name and value to an attribute string that
    /// can be sent to the C library.
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write.
    fn to_attr(&self) -> Result<String> {
        Ok(format!("{}", self))
    }
}

/// Trait to convert an attribute string to a typed value.
pub trait FromAttribute: FromStr + Sized {
    /// Converts a string attribute to a value type.
    fn from_attr(s: &str) -> Result<Self> {
        let val = Self::from_str(s).map_err(
            |_| Error::StringConversionError)?;
        Ok(val)
    }
}

/// Attribute conversion for the bool type.
///
/// The bool type needs a special implementation of the attribute conversion
/// trait because it's default Rust string counterparts are "true" and "false"
/// for true and false values respectively. However, sysfs expects these to be
/// "1" or "0".
impl ToAttribute for bool {
    fn to_attr(&self) -> Result<String> {
        Ok((if *self { "1" } else { "0" }).into())
    }
}

impl FromAttribute for bool {
    fn from_attr(s: &str) -> Result<bool> {
        Ok(if s.trim() == "0" { false } else { true })
    }
}

// Default trait implementations for the types in the IIO lib
impl ToAttribute for i64 {}
impl ToAttribute for f64 {}
impl ToAttribute for &str {}
impl ToAttribute for String {}

impl FromAttribute for i64 {}
impl FromAttribute for f64 {}
impl FromAttribute for String {}


// Callback from the C lib to extract the collection of all
// device-specific attributes. See attr_read_all().
pub(crate) unsafe extern "C" fn attr_read_all_cb(
    _chan: *mut ffi::iio_device,
    attr: *const c_char,
    val: *const c_char,
    _len: usize,
    pmap: *mut c_void,
) -> c_int {
    if attr.is_null() || val.is_null() || pmap.is_null() {
        return -1;
    }

    let attr = CStr::from_ptr(attr).to_string_lossy().to_string();
    // TODO: We could/should check val[len-1] == '\x0'
    let val = CStr::from_ptr(val).to_string_lossy().to_string();
    let map: &mut HashMap<String, String> = &mut *(pmap as *mut _);
    map.insert(attr, val);
    0
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

    #[test]
    fn string_to_attr_val() {
        let val: i32 = string_to_attr("123".to_string()).unwrap();
        assert_eq!(val, 123);

        let val = string_to_attr::<bool>("1".to_string()).unwrap();
        assert_eq!(val, true);

        let val: bool = string_to_attr(" 0 \n".to_string()).unwrap();
        assert_eq!(val, false);

        let val: String = string_to_attr("hello".to_string()).unwrap();
        assert_eq!(&val, "hello");
    }

    #[test]
    fn attr_val_to_string() {
        let s = attr_to_string(123).unwrap();
        assert_eq!(&s, "123");

        let s = attr_to_string(true).unwrap();
        assert_eq!(&s, "1");

        let s = attr_to_string(false).unwrap();
        assert_eq!(&s, "0");

        let s = attr_to_string("hello").unwrap();
        assert_eq!(&s, "hello");
    }
}
