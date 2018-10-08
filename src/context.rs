// libiio-sys/src/context.rs
//
//!
//!

use std::ffi::CString;
use std::os::raw::c_uint;

use nix::errno::{Errno};
use nix::Error::Sys as SysError;

use ffi;
use errors::*;
use device::*;

/// An Industrial I/O Context
pub struct Context {
    ctx: *mut ffi::iio_context,
}

impl Context {
    /// Tries to create a context from a local or remote IIO device.
    pub fn new() -> Result<Context> {
        let ctx = unsafe { ffi::iio_create_default_context() };
        if ctx.is_null() { bail!(SysError(Errno::last())); }
        Ok(Context { ctx, })
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
            //bail!(SysError(Errno::last()));
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

