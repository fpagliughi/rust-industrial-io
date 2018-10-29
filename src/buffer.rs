// libiio-sys/src/buffer.rs
//
// Copyright (c) 2018, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//! Industrial I/O Buffers
//!

use std::{mem, ptr};
use std::marker::PhantomData;

use nix::errno::{Errno};
use nix::Error::Sys as SysError;

use ffi;
use errors::*;
use device::*;
use channel::*;

/// An Industrial I/O input or output buffer
pub struct Buffer {
    pub(crate) buf: *mut ffi::iio_buffer,
}

impl Buffer {

    /// Get the device to which this buffer belongs
    pub fn device(&self) -> Device {
        // TODO: Check C API to make sure it's safe to convert to *mut
        let dev = unsafe { ffi::iio_buffer_get_device(self.buf) as *mut ffi::iio_device };
        Device { dev, }
    }

    /// Fetch more samples from the hardware
    ///
    /// This is only valid for input buffers
    pub fn refill(&mut self) -> Result<usize> {
        let n = unsafe { ffi::iio_buffer_refill(self.buf) };
        if n < 0 { bail!(SysError(Errno::last())); }
        Ok(n as usize)
    }

    /// Send the samples to the hardware.
    ///
    /// This is only valid for output buffers
    pub fn push(&mut self) -> Result<usize> {
        let n = unsafe { ffi::iio_buffer_push(self.buf) };
        if n < 0 { bail!(SysError(Errno::last())); }
        Ok(n as usize)
    }

    /// Send a given number of samples to the hardware.
    ///
    /// `n` The number of samples to send
    /// This is only valid for output buffers
    pub fn push_partial(&mut self, n: usize) -> Result<usize> {
        let n = unsafe { ffi::iio_buffer_push_partial(self.buf, n) };
        if n < 0 { bail!(SysError(Errno::last())); }
        Ok(n as usize)
    }

    /// Gets an iterator for the data from a channel.
    pub fn channel_iter<T>(&self, chan: &Channel) -> IntoIter<T> {
        unsafe {
            let begin = ffi::iio_buffer_first(self.buf, chan.chan) as *mut T;
            let end = ffi::iio_buffer_end(self.buf) as *const T;
            let ptr = begin;
            let step: isize = ffi::iio_buffer_step(self.buf)/mem::size_of::<T>() as isize;

            IntoIter {
                phantom: PhantomData,
                ptr,
                end,
                step,
            }
        }
    }
}

/// An iterator that moves channel data out of a buffer.
pub struct IntoIter<T> {
    phantom: PhantomData<T>,
    // Pointer to the current sample for a channel
    ptr: *const T,
    // Pointer to the end of the buffer
    end: *const T,
    // The offset to the next sample for the channel
    step: isize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        unsafe {
            if self.ptr as *const _ >= self.end {
                None
            }
            else {
                let prev = self.ptr;
                self.ptr = self.ptr.offset(self.step);
                Some(ptr::read(prev))
            }
        }
    }
}


