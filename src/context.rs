// libiio-sys/src/context.rs
//
// Copyright (c) 2018, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//! Industrial I/O Contexts.
//!

use std::time::Duration;
use std::ffi::CString;
use std::os::raw::c_uint;
use std::rc::Rc;

use nix::errno::{Errno};
use nix::Error::Sys as SysError;

use ffi;
use super::*;

/** An Industrial I/O Context
Since IIO doesn't provide any thread safety guarantees, this object cannot be Send or Sync.
This object maintains a reference counted pointer to the context object of the underlying library's iio_context object.
Once all references to the Context object have been dropped, the underlying iio_context will be destroyed.
This is done to make creation and use of a single Device more ergonomic by removing the need to manage the lifetime of the Context.
**/
#[derive(Debug,Clone)]
pub struct Context {
    raw: Rc<RawContext>,
}

/// RawContext holds a 
#[derive(Debug)]
struct RawContext {
    pub(crate) ctx: *mut ffi::iio_context
}

impl Drop for RawContext {
    fn drop(&mut self) {
        unsafe { ffi::iio_context_destroy(self.ctx) };
    }
}

impl Context {
    /// Tries to create a context from a local or remote IIO device.
    pub fn new() -> Result<Context> {
        let ctx = unsafe { ffi::iio_create_default_context() };
        if ctx.is_null() { bail!(SysError(Errno::last())); }
        Ok(Context { raw: Rc::new(RawContext{ ctx }) })
    }

    /// Get a description of the context
    pub fn description(&self) -> String {
        let pstr = unsafe { ffi::iio_context_get_description(self.raw.ctx) };
        cstring_opt(pstr).unwrap_or_default()
    }

    /// Gets the number of context-specific attributes
    pub fn num_attrs(&self) -> usize {
        let n = unsafe { ffi::iio_context_get_attrs_count(self.raw.ctx) };
        n as usize
    }

    /// Sets the timeout for I/O operations
    ///
    /// `timeout` The timeout. A value of zero specifies that no timeout
    /// should be used.
    pub fn set_timeout(&mut self, timeout: Duration) -> Result<()> {
        let timeout_ms: u64 = 1000 * timeout.as_secs() + u64::from(timeout.subsec_millis());
        let ret = unsafe { ffi::iio_context_set_timeout(self.raw.ctx, timeout_ms as c_uint) };
        if ret < 0 { bail!(SysError(Errno::last())); }
        Ok(())
    }

    /// Get the number of devices in the context
    pub fn num_devices(&self) -> usize {
        let n = unsafe { ffi::iio_context_get_devices_count(self.raw.ctx) };
        n as usize
    }

    /// Gets a device by index
    pub fn get_device(&self, idx: usize) -> Result<Device> {
        let dev = unsafe { ffi::iio_context_get_device(self.raw.ctx, idx as c_uint) };
        if dev.is_null() { bail!("Index out of range"); }
        Ok(Device { dev, ctx: self.clone() })
    }

    /// Try to find a device by name or ID
    /// `name` The name or ID of the device to find
    pub fn find_device(&self, name: &str) -> Option<Device> {
        let name = CString::new(name).unwrap();
        let dev = unsafe { ffi::iio_context_find_device(self.raw.ctx, name.as_ptr()) };
        if dev.is_null() {
            None
        }
        else {
            Some(Device { dev, ctx: self.clone() })
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

impl PartialEq for Context {
    /// Two contexts are the same if they refer to the same underlying
    /// object in the library.
    fn eq(&self, other: &Context) -> bool {
        self.raw.ctx == other.raw.ctx
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

