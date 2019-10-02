// libiio-sys/src/channel.rs
//
// Copyright (c) 2018, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//! Industrial I/O Channels
//!

use std::ffi::CString;
use std::os::raw::{c_longlong, c_uint};

use nix::errno::Errno;
use nix::Error::Sys as SysError;

use super::*;
use ffi;

#[derive(Debug, PartialEq)]
pub enum ChannelType {
    Input,
    Output,
}

/// An Industrial I/O Device Channel
pub struct Channel {
    pub(crate) chan: *mut ffi::iio_channel,
    #[allow(dead_code)]
    // looks like it's unused, but really it's holding the Device's lifetime for libiio safety.
    pub(crate) ctx: Context,
}

impl Channel {
    /// Retrieves the name of the channel (e.g. <b><i>vccint</i></b>)
    pub fn name(&self) -> Option<String> {
        let pstr = unsafe { ffi::iio_channel_get_name(self.chan) };
        cstring_opt(pstr)
    }

    /// @brief Retrieve the channel ID (e.g. <b><i>voltage0</i></b>)
    pub fn id(&self) -> Option<String> {
        let pstr = unsafe { ffi::iio_channel_get_id(self.chan) };
        cstring_opt(pstr)
    }

    /// Determines if the channel is a scan element
    ///
    /// A scan element is a channel that can generate samples (for an
    /// input  channel) or receive samples (for an output channel) after
    /// being enabled.
    pub fn is_scan_element(&self) -> bool {
        unsafe { ffi::iio_channel_is_scan_element(self.chan) }
    }

    /// Gets the number of context-specific attributes
    pub fn num_attrs(&self) -> usize {
        let n = unsafe { ffi::iio_channel_get_attrs_count(self.chan) };
        n as usize
    }

    /// Gets the channel-specific attribute at the index
    pub fn get_attr(&self, idx: usize) -> Result<String> {
        let pstr = unsafe { ffi::iio_channel_get_attr(self.chan, idx as c_uint) };
        cstring_opt(pstr).ok_or_else(|| "Invalid index".into())
    }

    /// Reads a channel-specific attribute as a boolean
    /// `attr` The name of the attribute
    pub fn attr_read_bool(&self, attr: &str) -> Result<bool> {
        let mut val: bool = false;
        let attr = CString::new(attr).unwrap();
        unsafe {
            if ffi::iio_channel_attr_read_bool(self.chan, attr.as_ptr(), &mut val) < 0 {
                bail!(SysError(Errno::last()));
            }
        }
        Ok(val)
    }

    /// Reads a channel-specific attribute as an integer (i64)
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_int(&self, attr: &str) -> Result<i64> {
        let mut val: c_longlong = 0;
        let attr = CString::new(attr).unwrap();
        unsafe {
            if ffi::iio_channel_attr_read_longlong(self.chan, attr.as_ptr(), &mut val) < 0 {
                bail!(SysError(Errno::last()));
            }
        }
        Ok(val as i64)
    }

    /// Reads a channel-specific attribute as a floating-point (f64) number
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_float(&self, attr: &str) -> Result<f64> {
        let mut val: f64 = 0.0;
        let attr = CString::new(attr).unwrap();
        unsafe {
            if ffi::iio_channel_attr_read_double(self.chan, attr.as_ptr(), &mut val) < 0 {
                bail!(SysError(Errno::last()));
            }
        }
        Ok(val)
    }

    /// Writes a channel-specific attribute as a boolean
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_bool(&self, attr: &str, val: bool) -> Result<()> {
        let attr = CString::new(attr).unwrap();
        unsafe {
            if ffi::iio_channel_attr_write_bool(self.chan, attr.as_ptr(), val) < 0 {
                bail!(SysError(Errno::last()));
            }
        }
        Ok(())
    }

    /// Writes a channel-specific attribute as an integer (i64)
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_int(&self, attr: &str, val: i64) -> Result<()> {
        let attr = CString::new(attr).unwrap();
        unsafe {
            if ffi::iio_channel_attr_write_longlong(self.chan, attr.as_ptr(), val) < 0 {
                bail!(SysError(Errno::last()));
            }
        }
        Ok(())
    }

    /// Writes a channel-specific attribute as a floating-point (f64) number
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_float(&self, attr: &str, val: f64) -> Result<()> {
        let attr = CString::new(attr).unwrap();
        unsafe {
            if ffi::iio_channel_attr_write_double(self.chan, attr.as_ptr(), val) < 0 {
                bail!(SysError(Errno::last()));
            }
        }
        Ok(())
    }

    /// Gets an iterator for the attributes of the channel
    pub fn attrs(&self) -> AttrIterator {
        AttrIterator { chan: self, idx: 0 }
    }

    /// Enable the channel
    ///
    /// Before creating a buffer, at least one channel of the device
    /// must be enabled.
    pub fn enable(&mut self) {
        unsafe { ffi::iio_channel_enable(self.chan) };
    }

    /// Disables the channel
    pub fn disable(&mut self) {
        unsafe { ffi::iio_channel_disable(self.chan) };
    }

    /// Determines if the channel is enabled
    pub fn is_enabled(&self) -> bool {
        unsafe { ffi::iio_channel_is_enabled(self.chan) }
    }
}

pub struct AttrIterator<'a> {
    chan: &'a Channel,
    idx: usize,
}

impl<'a> Iterator for AttrIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.chan.get_attr(self.idx) {
            Ok(name) => {
                self.idx += 1;
                Some(name)
            }
            Err(_) => None,
        }
    }
}
