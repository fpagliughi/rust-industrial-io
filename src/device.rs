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

use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::{c_int, c_longlong, c_uint, c_void};
use std::{ptr, str};

use nix::errno::Errno;

use super::*;
use crate::ffi;

/// An Industrial I/O Device
///
/// This can not be created directly. It is obtained from a context.
#[derive(Debug)]
pub struct Device {
    /// Pointer to the underlying device object.
    pub(crate) dev: *mut ffi::iio_device,
    /// The IIO context containing the device.
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

    /// Determines if the device is capable of buffered I/O.
    /// This is true if any of the channels are scan elements.
    pub fn is_buffer_capable(&self) -> bool {
        // This "trick" is from C lib 'iio_info.c'
        for chan in self.channels() {
            if chan.is_scan_element() {
                return true;
            }
        }
        false
    }

    /// Determines whether the device is a trigger
    pub fn is_trigger(&self) -> bool {
        unsafe { ffi::iio_device_is_trigger(self.dev) }
    }

    /// Associate a trigger for this device.
    /// `trigger` The device to be used as a trigger.
    pub fn set_trigger(&mut self, trigger: &Device) -> Result<()> {
        let ret = unsafe { ffi::iio_device_set_trigger(self.dev, trigger.dev) };
        sys_result(ret, ())
    }

    /// Removes the trigger from the device.
    pub fn remove_trigger(&mut self) -> Result<()> {
        let ret = unsafe { ffi::iio_device_set_trigger(self.dev, ptr::null()) };
        sys_result(ret, ())
    }

    // ----- Attributes -----

    /// Gets the number of device-specific attributes
    pub fn num_attrs(&self) -> usize {
        unsafe { ffi::iio_device_get_attrs_count(self.dev) as usize }
    }

    /// Gets the name of the device-specific attribute at the index
    pub fn get_attr(&self, idx: usize) -> Result<String> {
        let pstr = unsafe { ffi::iio_device_get_attr(self.dev, idx as c_uint) };
        cstring_opt(pstr).ok_or(Error::InvalidIndex)
    }

    /// Try to find a device-specific attribute by its name
    pub fn find_attr(&self, name: &str) -> Option<String> {
        let cname = cstring_or_bail!(name);
        let pstr = unsafe { ffi::iio_device_find_attr(self.dev, cname.as_ptr()) };
        cstring_opt(pstr)
    }

    /// Determines if a buffer-specific attribute exists
    pub fn has_attr(&self, name: &str) -> bool {
        let cname = cstring_or_bail_false!(name);
        let pstr = unsafe { ffi::iio_device_find_attr(self.dev, cname.as_ptr()) };
        !pstr.is_null()
    }

    /// Reads a device-specific attribute as a boolean
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_bool(&self, attr: &str) -> Result<bool> {
        let mut val: bool = false;
        let attr = CString::new(attr)?;
        let ret = unsafe { ffi::iio_device_attr_read_bool(self.dev, attr.as_ptr(), &mut val) };
        sys_result(ret, val)
    }

    /// Reads a device-specific attribute as an integer (i64)
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_int(&self, attr: &str) -> Result<i64> {
        let mut val: c_longlong = 0;
        let attr = CString::new(attr)?;
        let ret = unsafe { ffi::iio_device_attr_read_longlong(self.dev, attr.as_ptr(), &mut val) };
        sys_result(ret, val as i64)
    }

    /// Reads a device-specific attribute as a floating-point (f64) number
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_float(&self, attr: &str) -> Result<f64> {
        let mut val: f64 = 0.0;
        let attr = CString::new(attr)?;
        let ret = unsafe { ffi::iio_device_attr_read_double(self.dev, attr.as_ptr(), &mut val) };
        sys_result(ret, val)
    }

    // Callback from the C lib to extract the collection of all
    // device-specific attributes. See attr_read_all().
    unsafe extern "C" fn attr_read_all_cb(
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

    /// Reads all the device-specific attributes.
    /// This is especially useful when using the network backend to
    /// retrieve all the attributes with a single call.
    pub fn attr_read_all(&self) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        let pmap = &mut map as *mut _ as *mut c_void;
        let ret = unsafe {
            ffi::iio_device_attr_read_all(self.dev, Some(Device::attr_read_all_cb), pmap)
        };
        sys_result(ret, map)
    }

    /// Writes a device-specific attribute as a boolean
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_bool(&self, attr: &str, val: bool) -> Result<()> {
        let attr = CString::new(attr)?;
        let ret = unsafe { ffi::iio_device_attr_write_bool(self.dev, attr.as_ptr(), val) };
        sys_result(ret, ())
    }

    /// Writes a device-specific attribute as an integer (i64)
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_int(&self, attr: &str, val: i64) -> Result<()> {
        let attr = CString::new(attr)?;
        let ret = unsafe { ffi::iio_device_attr_write_longlong(self.dev, attr.as_ptr(), val) };
        sys_result(ret, ())
    }

    /// Writes a device-specific attribute as a floating-point (f64) number
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_float(&self, attr: &str, val: f64) -> Result<()> {
        let attr = CString::new(attr)?;
        let ret = unsafe { ffi::iio_device_attr_write_double(self.dev, attr.as_ptr(), val) };
        sys_result(ret, ())
    }

    /// Gets an iterator for the attributes in the device
    pub fn attributes(&self) -> AttrIterator {
        AttrIterator { dev: self, idx: 0 }
    }

    // ----- Channels -----

    /// Gets the number of channels on the device
    pub fn num_channels(&self) -> usize {
        unsafe { ffi::iio_device_get_channels_count(self.dev) as usize }
    }

    /// Gets a channel by index
    pub fn get_channel(&self, idx: usize) -> Result<Channel> {
        let chan = unsafe { ffi::iio_device_get_channel(self.dev, idx as c_uint) };
        if chan.is_null() {
            return Err(Error::InvalidIndex);
        }
        Ok(Channel {
            chan,
            ctx: self.context(),
        })
    }

    /// Try to find a channel by its name or ID
    pub fn find_channel(&self, name: &str, is_output: bool) -> Option<Channel> {
        let cname = cstring_or_bail!(name);
        let chan = unsafe { ffi::iio_device_find_channel(self.dev, cname.as_ptr(), is_output) };

        if chan.is_null() {
            None
        }
        else {
            Some(Channel {
                chan,
                ctx: self.context(),
            })
        }
    }

    /// Gets an iterator for the channels in the device
    pub fn channels(&self) -> ChannelIterator {
        ChannelIterator { dev: self, idx: 0 }
    }

    // ----- Buffer Functions -----

    /// Creates a buffer for the device.
    ///
    /// `sample_count` The number of samples the buffer should hold
    /// `cyclic` Whether to enable cyclic mode.
    pub fn create_buffer(&self, sample_count: usize, cyclic: bool) -> Result<Buffer> {
        let buf = unsafe { ffi::iio_device_create_buffer(self.dev, sample_count, cyclic) };
        if buf.is_null() {
            return Err(Errno::last().into());
        }
        Ok(Buffer {
            buf,
            cap: sample_count,
            ctx: self.context(),
        })
    }

    /// Gets the number of buffer-specific attributes
    pub fn num_buffer_attrs(&self) -> usize {
        unsafe { ffi::iio_device_get_buffer_attrs_count(self.dev) as usize }
    }

    /// Gets the name of the buffer-specific attribute at the index
    pub fn get_buffer_attr(&self, idx: usize) -> Result<String> {
        let pstr = unsafe { ffi::iio_device_get_buffer_attr(self.dev, idx as c_uint) };
        cstring_opt(pstr).ok_or(Error::InvalidIndex)
    }

    /// Try to find a buffer-specific attribute by its name
    pub fn find_buffer_attr(&self, name: &str) -> Option<String> {
        let cname = cstring_or_bail!(name);
        let pstr = unsafe { ffi::iio_device_find_buffer_attr(self.dev, cname.as_ptr()) };
        cstring_opt(pstr)
    }

    /// Determines if a buffer-specific attribute exists
    pub fn has_buffer_attr(&self, name: &str) -> bool {
        let cname = cstring_or_bail_false!(name);
        let pstr = unsafe { ffi::iio_device_find_buffer_attr(self.dev, cname.as_ptr()) };
        !pstr.is_null()
    }

    /// Reads a buffer-specific attribute as a boolean
    ///
    /// `attr` The name of the attribute
    pub fn buffer_attr_read_bool(&self, attr: &str) -> Result<bool> {
        let mut val: bool = false;
        let attr = CString::new(attr)?;
        let ret =
            unsafe { ffi::iio_device_buffer_attr_read_bool(self.dev, attr.as_ptr(), &mut val) };
        sys_result(ret, val)
    }

    /// Reads a buffer-specific attribute as an integer (i64)
    ///
    /// `attr` The name of the attribute
    pub fn buffer_attr_read_int(&self, attr: &str) -> Result<i64> {
        let mut val: c_longlong = 0;
        let attr = CString::new(attr)?;
        let ret =
            unsafe { ffi::iio_device_buffer_attr_read_longlong(self.dev, attr.as_ptr(), &mut val) };
        sys_result(ret, val as i64)
    }

    /// Reads a buffer-specific attribute as a floating-point (f64) number
    ///
    /// `attr` The name of the attribute
    pub fn buffer_attr_read_float(&self, attr: &str) -> Result<f64> {
        let mut val: f64 = 0.0;
        let attr = CString::new(attr)?;
        let ret =
            unsafe { ffi::iio_device_buffer_attr_read_double(self.dev, attr.as_ptr(), &mut val) };
        sys_result(ret, val)
    }

    /// Reads all the buffer-specific attributes.
    /// This is especially useful when using the network backend to
    /// retrieve all the attributes with a single call.
    pub fn buffer_attr_read_all(&self) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        let pmap = &mut map as *mut _ as *mut c_void;
        let ret = unsafe {
            ffi::iio_device_buffer_attr_read_all(self.dev, Some(Device::attr_read_all_cb), pmap)
        };
        sys_result(ret, map)
    }

    /// Writes a buffer-specific attribute as a boolean
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn buffer_attr_write_bool(&self, attr: &str, val: bool) -> Result<()> {
        let attr = CString::new(attr)?;
        let ret = unsafe { ffi::iio_device_buffer_attr_write_bool(self.dev, attr.as_ptr(), val) };
        sys_result(ret, ())
    }

    /// Writes a device-specific attribute as an integer (i64)
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn buffer_attr_write_int(&self, attr: &str, val: i64) -> Result<()> {
        let attr = CString::new(attr)?;
        let ret =
            unsafe { ffi::iio_device_buffer_attr_write_longlong(self.dev, attr.as_ptr(), val) };
        sys_result(ret, ())
    }

    /// Writes a device-specific attribute as a floating-point (f64) number
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn buffer_attr_write_float(&self, attr: &str, val: f64) -> Result<()> {
        let attr = CString::new(attr)?;
        let ret = unsafe { ffi::iio_device_buffer_attr_write_double(self.dev, attr.as_ptr(), val) };
        sys_result(ret, ())
    }

    /// Gets an iterator for the buffer attributes in the device
    pub fn buffer_attributes(&self) -> BufferAttrIterator {
        BufferAttrIterator { dev: self, idx: 0 }
    }

    /// Set the number of kernel buffers for the device.
    pub fn set_num_kernel_buffers(&self, n: u32) -> Result<()> {
        let ret = unsafe { ffi::iio_device_set_kernel_buffers_count(self.dev, n as c_uint) };
        sys_result(ret, ())
    }

    // ----- Low-level & Debug functions -----

    /// Gets the current sample size, in bytes.
    /// This gets the number of bytes requires to store the samples,
    /// based on the the channels that are currently enabled.
    pub fn sample_size(&self) -> Result<usize> {
        let ret = unsafe { ffi::iio_device_get_sample_size(self.dev) };
        sys_result(ret as i32, ret as usize)
    }

    /// Gets the value of a hardware register
    pub fn reg_read(&self, addr: u32) -> Result<u32> {
        let mut val: u32 = 0;
        let ret = unsafe { ffi::iio_device_reg_read(self.dev, addr, &mut val) };
        sys_result(ret as i32, val)
    }

    /// Sets the value of a hardware register
    pub fn reg_write(&self, addr: u32, val: u32) -> Result<()> {
        let ret = unsafe { ffi::iio_device_reg_write(self.dev, addr, val) };
        sys_result(ret as i32, ())
    }
}

impl PartialEq for Device {
    /// Two devices are the same if they refer to the same underlying
    /// object in the library.
    fn eq(&self, other: &Device) -> bool {
        self.dev == other.dev
    }
}

/// Iterator over the Channels in a Device
#[derive(Debug)]
pub struct ChannelIterator<'a> {
    /// Reference to the Device that we're scanning for Channels
    dev: &'a Device,
    /// Index for the next Channel from the iterator.
    idx: usize,
}

impl<'a> Iterator for ChannelIterator<'a> {
    type Item = Channel;

    fn next(&mut self) -> Option<Self::Item> {
        match self.dev.get_channel(self.idx) {
            Ok(chan) => {
                self.idx += 1;
                Some(chan)
            }
            Err(_) => None,
        }
    }
}

/// Iterator over the attributes in a Device
#[derive(Debug)]
pub struct AttrIterator<'a> {
    /// Reference to the Device that we're scanning for attributes
    dev: &'a Device,
    /// Index for the next Device attribute from the Iterator.
    idx: usize,
}

impl<'a> Iterator for AttrIterator<'a> {
    type Item = String;

    /// Gets the next Device attribute from the iterator
    fn next(&mut self) -> Option<Self::Item> {
        match self.dev.get_attr(self.idx) {
            Ok(name) => {
                self.idx += 1;
                Some(name)
            }
            Err(_) => None,
        }
    }
}

/// Iterator over the buffer attributes in a Device
#[derive(Debug)]
pub struct BufferAttrIterator<'a> {
    /// Reference to the Buffer that we're scanning for attributes
    dev: &'a Device,
    /// Index to the next Buffer attribute from the iterator
    idx: usize,
}

impl<'a> Iterator for BufferAttrIterator<'a> {
    type Item = String;

    /// Gets the next Buffer attribute from the iterator
    fn next(&mut self) -> Option<Self::Item> {
        match self.dev.get_buffer_attr(self.idx) {
            Ok(name) => {
                self.idx += 1;
                Some(name)
            }
            Err(_) => None,
        }
    }
}

// --------------------------------------------------------------------------
//                              Unit Tests
// --------------------------------------------------------------------------

// Note: These tests assume that the IIO Dummy kernel module is loaded
// locally with a device created. See the `load_dummy.sh` script.

#[cfg(test)]
mod tests {
    use super::*;

    // Make sure we get a device
    #[test]
    fn get_device() {
        let ctx = Context::new().unwrap();
        let dev = ctx.get_device(0);
        assert!(dev.is_ok());

        let dev = dev.unwrap();
        let id = dev.id().unwrap();
        assert!(!id.is_empty());
    }

    // See that attr iterator gets the correct number of attributes
    #[test]
    fn attr_iterator_count() {
        let ctx = Context::new().unwrap();
        let dev = ctx.get_device(0).unwrap();

        let n = dev.num_attrs();
        assert!(n != 0);
        assert!(dev.attributes().count() == n);
    }
}

