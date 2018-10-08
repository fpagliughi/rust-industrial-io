// libiio-sys/src/device.rs
//
//!
//!

use std::ffi::{CString, CStr};

use nix::errno::{Errno};
use nix::Error::Sys as SysError;

use ffi;
use errors::*;
use channel::*;

/// An Industrial I/O Device
///
/// This can not be created directly. It is obtained from a context.
pub struct Device {
    pub(crate) dev: *mut ffi::iio_device,
}

impl Device {
    /// Gets the name of the device
    pub fn name(&self) -> Option<String> {
        let pstr = unsafe { ffi::iio_device_get_name(self.dev) };
        if pstr.is_null() {
            None
        }
        else {
            let name = unsafe { CStr::from_ptr(pstr) };
            Some(name.to_str().unwrap_or_default().to_string())
        }
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

    /// Gets the number of channels on the device
    pub fn num_channels(&self) -> usize {
        let n = unsafe { ffi::iio_device_get_channels_count(self.dev) };
        n as usize
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

}

impl PartialEq for Device {
    /// Two devices are the same if they refer to the same underlying
    /// object in the library.
    fn eq(&self, other: &Device) -> bool {
        self.dev == other.dev
    }
}

