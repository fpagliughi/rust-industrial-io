// libiio-sys/src/device.rs
//
//!
//!

use std::ffi::{CString, CStr};
use std::os::raw::{c_uint, c_longlong};

use nix::errno::{Errno};
use nix::Error::Sys as SysError;

use ffi;
use super::*;
use errors::*;
use channel::*;

/// An Industrial I/O Device
///
/// This can not be created directly. It is obtained from a context.
pub struct Device {
    pub(crate) dev: *mut ffi::iio_device,
}

impl Device {
    /// Gets the context to which the device belongs
    pub fn context(&self) -> Context {
        let ctx = unsafe { ffi::iio_device_get_context(self.dev) as *mut ffi::iio_context };
        if ctx.is_null() { panic!("Unexpected NULL context"); }
        Context { ctx, }
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
        cstring_opt(pstr).ok_or("Invalid index".into())
    }

    /// Reads a device-specific attribute as a boolean
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_bool(&self, attr: &str) -> Result<bool> {
        let mut val: bool = false;
        let attr = CString::new(attr).unwrap();
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
        let attr = CString::new(attr).unwrap();
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
        let attr = CString::new(attr).unwrap();
        unsafe {
            if ffi::iio_device_attr_read_double(self.dev, attr.as_ptr(), &mut val) < 0 {
                bail!(SysError(Errno::last()));
            }
        }
        Ok(val)
    }

    /// Writes a device-specific attribute as a boolean
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_bool(&self, attr: &str, val: bool) -> Result<()> {
        let attr = CString::new(attr).unwrap();
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
        let attr = CString::new(attr).unwrap();
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
        let attr = CString::new(attr).unwrap();
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
        Ok(Channel { chan, })
    }

    /// Try to find a channel by its name or ID
    pub fn find_channel(&self, name: &str, chan_type: ChannelType) -> Option<Channel> {
        let name = CString::new(name).unwrap();
        let is_output = chan_type == ChannelType::Output;
        let chan = unsafe { ffi::iio_device_find_channel(self.dev, name.as_ptr(), is_output) };

        if chan.is_null() {
            None
        }
        else {
            Some(Channel { chan, })
        }
    }

    /// Gets an iterator for the channels in the device
    pub fn channels(&self) -> ChannelIterator {
        ChannelIterator {
            dev: self,
            idx: 0,
        }
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

