// libiio-sys/src/device.rs
//
// Copyright (c) 2018, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//! Industrial I/O Devices
//!

use std::str;
use std::ffi::CString;
use std::os::raw::{c_void, c_int, c_uint, c_longlong};
use std::collections::HashMap;

use nix::errno::{Errno};
use nix::Error::Sys as SysError;

use ffi;
use super::*;

/// An Industrial I/O Device
///
/// This can not be created directly. It is obtained from a context.
pub struct Device {
    pub(crate) dev: *mut ffi::iio_device,
    pub(crate) ctx: Context,
}

impl Device {
    /// Gets the context to which the device belongs
    pub fn context(&self) -> Context {
        self.ctx.clone()
    }

    /// Gets the device ID (e.g. <b><i>iio:device0</i></b>)
    pub fn id(&self) -> Option<String> {
        let pstr = unsafe { ffi::iio_device_get_id(self.dev) };
        cstring_opt(pstr)
    }

    /// Gets the name of the device
    pub fn name(&self) -> Option<String> {
        let pstr = unsafe { ffi::iio_device_get_name(self.dev) };
        cstring_opt(pstr)
    }

    /// Determines whether the device is a trigger
    pub fn is_trigger(&self) -> bool {
        unsafe { ffi::iio_device_is_trigger(self.dev) }
    }

    /// Associate a trigger for this device.
    /// `trigger` The device to be used as a trigger.
    pub fn set_trigger(&mut self, trigger: &Device) -> Result<()> {
        let ret = unsafe { ffi::iio_device_set_trigger(self.dev, trigger.dev) };
        if ret < 0 { bail!(SysError(Errno::last())); }
        Ok(())
    }

    /// Gets the number of device-specific attributes
    pub fn num_attrs(&self) -> usize {
        let n = unsafe { ffi::iio_device_get_attrs_count(self.dev) };
        n as usize
    }

    /// Gets the name of the device-specific attribute at the index
    pub fn get_attr(&self, idx: usize) -> Result<String> {
        let pstr = unsafe { ffi::iio_device_get_attr(self.dev, idx as c_uint) };
        cstring_opt(pstr).ok_or_else(|| "Invalid index".into())
    }

    /// Reads a device-specific attribute as a boolean
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_bool(&self, attr: &str) -> Result<bool> {
        let mut val: bool = false;
        let attr = CString::new(attr)?;
        unsafe {
            if ffi::iio_device_attr_read_bool(self.dev, attr.as_ptr(), &mut val) < 0 {
                bail!(SysError(Errno::last()));
            }
        }
        Ok(val)
    }

    /// Reads a device-specific attribute as an integer (i64)
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_int(&self, attr: &str) -> Result<i64> {
        let mut val: c_longlong = 0;
        let attr = CString::new(attr)?;
        unsafe {
            if ffi::iio_device_attr_read_longlong(self.dev, attr.as_ptr(), &mut val) < 0 {
                bail!(SysError(Errno::last()));
            }
        }
        Ok(val as i64)
    }

    /// Reads a device-specific attribute as a floating-point (f64) number
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_float(&self, attr: &str) -> Result<f64> {
        let mut val: f64 = 0.0;
        let attr = CString::new(attr)?;
        unsafe {
            if ffi::iio_device_attr_read_double(self.dev, attr.as_ptr(), &mut val) < 0 {
                bail!(SysError(Errno::last()));
            }
        }
        Ok(val)
    }


    // Callback from the C lib to extract the collection of all
    // device-specific attributes. See attr_read_all().
    unsafe extern "C" fn attr_read_all_cb(_chan: *mut ffi::iio_device,
                                          attr: *const c_char,
                                          val: *const c_char, _len: usize,
                                          pmap: *mut c_void) -> c_int {
        if attr.is_null() || val.is_null() || pmap.is_null() {
            return -1;
        }

        let attr = CStr::from_ptr(attr).to_string_lossy().to_string();
        // TODO: We could/should check val[len-1] == '\x0'
        let val = CStr::from_ptr(val).to_string_lossy().to_string();
        let map: &mut HashMap<String,String> = &mut *(pmap as *mut _);
        map.insert(attr, val);
        0
    }

    /// Reads all the device-specific attributes.
    /// This is especially useful when using the network backend to
    /// retrieve all the attributes with a single call.
    pub fn attr_read_all(&self) -> Result<HashMap<String,String>> {
        let mut map = HashMap::new();
        let pmap = &mut map as *mut _ as *mut c_void;
        unsafe {
            let ret = ffi::iio_device_attr_read_all(self.dev, Some(Device::attr_read_all_cb), pmap);
            if ret < 0 { bail!(SysError(Errno::last())); }
        }
        Ok(map)
    }

    /// Writes a device-specific attribute as a boolean
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_bool(&self, attr: &str, val: bool) -> Result<()> {
        let attr = CString::new(attr)?;
        unsafe {
            if ffi::iio_device_attr_write_bool(self.dev, attr.as_ptr(), val) < 0 {
                bail!(SysError(Errno::last()));
            }
        }
        Ok(())
    }

    /// Writes a device-specific attribute as an integer (i64)
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_int(&self, attr: &str, val: i64) -> Result<()> {
        let attr = CString::new(attr)?;
        unsafe {
            if ffi::iio_device_attr_write_longlong(self.dev, attr.as_ptr(), val) < 0 {
                bail!(SysError(Errno::last()));
            }
        }
        Ok(())
    }

    /// Writes a device-specific attribute as a floating-point (f64) number
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_float(&self, attr: &str, val: f64) -> Result<()> {
        let attr = CString::new(attr)?;
        unsafe {
            if ffi::iio_device_attr_write_double(self.dev, attr.as_ptr(), val) < 0 {
                bail!(SysError(Errno::last()));
            }
        }
        Ok(())
    }

    /// Gets the number of channels on the device
    pub fn num_channels(&self) -> usize {
        let n = unsafe { ffi::iio_device_get_channels_count(self.dev) };
        n as usize
    }

    /// Gets a channel by index
    pub fn get_channel(&self, idx: usize) -> Result<Channel> {
        let chan = unsafe { ffi::iio_device_get_channel(self.dev, idx as c_uint) };
        if chan.is_null() { bail!("Index out of range"); }
        Ok(Channel { chan, ctx: self.context() })
    }

    /// Try to find a channel by its name or ID
    pub fn find_channel(&self, name: &str, is_output: bool) -> Option<Channel> {
        let name = CString::new(name).unwrap();
        let chan = unsafe { ffi::iio_device_find_channel(self.dev, name.as_ptr(), is_output) };

        if chan.is_null() {
            None
        }
        else {
            Some(Channel { chan, ctx: self.context() })
        }
    }

    /// Gets an iterator for the channels in the device
    pub fn channels(&self) -> ChannelIterator {
        ChannelIterator {
            dev: self,
            idx: 0,
        }
    }

    /// Creates a buffer for the device.
    ///
    /// `sample_count` The number of samples the buffer should hold
    /// `cyclic` Whether to enable cyclic mode.
    pub fn create_buffer(&self, sample_count: usize, cyclic: bool) -> Result<Buffer> {
        let buf = unsafe { ffi::iio_device_create_buffer(self.dev, sample_count, cyclic) };
        if buf.is_null() { bail!(SysError(Errno::last())); }
        Ok(Buffer { buf, ctx: self.context() })
    }
}

impl PartialEq for Device {
    /// Two devices are the same if they refer to the same underlying
    /// object in the library.
    fn eq(&self, other: &Device) -> bool {
        self.dev == other.dev
    }
}

pub struct ChannelIterator<'a> {
    dev: &'a Device,
    idx: usize,
}

impl<'a> Iterator for ChannelIterator<'a> {
    type Item = Channel;

    fn next(&mut self) -> Option<Self::Item> {
        match self.dev.get_channel(self.idx) {
            Ok(chan) => {
                self.idx += 1;
                Some(chan)
            },
            Err(_) => None
        }
    }
}

pub struct AttrIterator<'a> {
    dev: &'a Device,
    idx: usize,
}

impl<'a> Iterator for AttrIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.dev.get_attr(self.idx) {
            Ok(name) => {
                self.idx += 1;
                Some(name)
            },
            Err(_) => None
        }
    }
}

