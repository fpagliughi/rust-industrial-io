// libiio-sys/src/scan_context.rs
//
// Copyright (c) 2023, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//! Scan context to get information about the available backends.

use crate::{cstring_opt, ffi, Error, Result};
use nix::errno::Errno;
use std::ffi::CString;

/// Scan context to get information about available contexts.
#[derive(Debug)]
pub struct ScanContext {
    /// Pointer to a libiio scan_block object
    pub(crate) ctx: *mut ffi::iio_scan_block,
}

impl ScanContext {
    /// Creates a scan context for the specified backend.
    /// The backend can be "local", "ip", or "usb".
    pub fn new(backend: &str) -> Result<Self> {
        let backend = CString::new(backend)?;
        let ctx = unsafe { ffi::iio_create_scan_block(backend.as_ptr(), 0) };
        if ctx.is_null() {
            return Err(Error::from(Errno::last()))
        }
        Ok(Self { ctx })
    }

    /// Creates a scan context for the USB backend.
    pub fn new_local() -> Result<Self> {
        Self::new("local")
    }

    /// Creates a scan context for the USB backend.
    pub fn new_network() -> Result<Self> {
        Self::new("ip")
    }

    /// Creates a scan context for the USB backend.
    pub fn new_usb() -> Result<Self> {
        Self::new("usb")
    }

    /// Gets the number of contexts in this backend
    pub fn len(&self) -> usize {
        let n = unsafe { ffi::iio_scan_block_scan(self.ctx) };
        if n < 0 { 0 } else { n as usize }
    }

    /// Gets an iterator to the contexts
    pub fn iter(&self) -> ScanContextIterator {
        ScanContextIterator { ctx: self, idx: 0 }
    }
}

impl Drop for ScanContext {
    /// Dropping destroys the underlying C scan context.
    fn drop(&mut self) {
        unsafe { ffi::iio_scan_block_destroy(self.ctx) };
    }
}

/// Iterator over the info in a ScanContext
#[derive(Debug)]
pub struct ScanContextIterator<'a> {
    /// Reference to the scan context that we're iterating through
    ctx: &'a ScanContext,
    /// Index for the next block from the iterator.
    idx: u32,
}

impl<'a> Iterator for ScanContextIterator<'a> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        let info = unsafe { ffi::iio_scan_block_get_info(self.ctx.ctx, self.idx) };
        if info.is_null() {
            None
        }
        else {
            let uri = cstring_opt(unsafe { ffi::iio_context_info_get_uri(info) })?;
            let descr = cstring_opt(unsafe { ffi::iio_context_info_get_description(info) })?;
            self.idx += 1;
            Some((uri, descr))
        }
    }
}

