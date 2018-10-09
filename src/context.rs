// libiio-sys/src/context.rs
//
//!
//!

use std::time::Duration;
use std::ffi::CString;
use std::os::raw::c_uint;

use nix::errno::{Errno};
use nix::Error::Sys as SysError;

use ffi;
use super::*;
use errors::*;
use device::*;

/// An Industrial I/O Context
#[derive(Debug)]
pub struct Context {
    pub(crate) ctx: *mut ffi::iio_context,
}

impl Context {
    /// Tries to create a context from a local or remote IIO device.
    pub fn new() -> Result<Context> {
        let ctx = unsafe { ffi::iio_create_default_context() };
        if ctx.is_null() { bail!(SysError(Errno::last())); }
        Ok(Context { ctx, })
    }

    /// Get a description of the context
    pub fn description(&self) -> String {
        let pstr = unsafe { ffi::iio_context_get_description(self.ctx) };
        cstring_opt(pstr).unwrap_or_default()
    }

    /// Gets the number of context-specific attributes
    pub fn num_attrs(&self) -> usize {
        let n = unsafe { ffi::iio_context_get_attrs_count(self.ctx) };
        n as usize
    }

    /// Sets the timeout for I/O operations
    ///
    /// `timeout` The timeout. A value of zero specifies that no timeout
    /// should be used.
    pub fn set_timeout(&mut self, timeout: Duration) -> Result<()> {
        let timeout_ms: u64 = 1000 * timeout.as_secs() + timeout.subsec_millis() as u64;
        let ret = unsafe { ffi::iio_context_set_timeout(self.ctx, timeout_ms as c_uint) };
        if ret < 0 { bail!(SysError(Errno::last())); }
        Ok(())
    }

    /// Get the number of devices in the context
    pub fn num_devices(&self) -> usize {
        let n = unsafe { ffi::iio_context_get_devices_count(self.ctx) };
        n as usize
    }

    /// Gets a device by index
    pub fn get_device(&self, idx: usize) -> Result<Device> {
        let dev = unsafe { ffi::iio_context_get_device(self.ctx, idx as c_uint) };
        if dev.is_null() { bail!("Index out of range"); }
        Ok(Device { dev, })
    }

    /// Try to find a device by name or ID
    /// `name` The name or ID of the device to find
    pub fn find_device(&self, name: &str) -> Option<Device> {
        let name = CString::new(name).unwrap();
        let dev = unsafe { ffi::iio_context_find_device(self.ctx, name.as_ptr()) };
        if dev.is_null() {
            None
        }
        else {
            Some(Device { dev, })
        }
    }

    /// Gets an iterator for all the devices in the context.
    pub fn devices(&self) -> DeviceIterator {
        DeviceIterator {
            ctx: self,
            idx: 0,
        }
    }

    /// Destroy the context
    ///
    /// This consumes the context to destroy the instance.
    pub fn destroy(self) {}
}

impl Clone for Context {
    fn clone(&self) -> Self {
        let ctx = unsafe { ffi::iio_context_clone(self.ctx) };
        if ctx.is_null() { panic!("Failed context clone"); }
        Context { ctx, }
    }
}

impl PartialEq for Context {
    /// Two contexts are the same if they refer to the same underlying
    /// object in the library.
    fn eq(&self, other: &Context) -> bool {
        self.ctx == other.ctx
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { ffi::iio_context_destroy(self.ctx) };
    }
}

pub struct DeviceIterator<'a> {
    ctx: &'a Context,
    idx: usize,
}

impl<'a> Iterator for DeviceIterator<'a> {
    type Item = Device;

    fn next(&mut self) -> Option<Self::Item> {
        match self.ctx.get_device(self.idx) {
            Ok(dev) => {
                self.idx += 1;
                Some(dev)
            },
            Err(_) => None
        }
    }
}

