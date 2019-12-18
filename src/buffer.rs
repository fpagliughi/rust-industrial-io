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
use std::os::raw::c_int;
use std::marker::PhantomData;

use ffi;
use super::*;

/// An Industrial I/O input or output buffer
pub struct Buffer {
    pub(crate) buf: *mut ffi::iio_buffer,
    #[allow(dead_code)] // this holds the refcount for libiio
    pub(crate) ctx: Context,
}

impl Buffer {
    /// Gets a pollable file descriptor for the buffer.
    /// This can be used to determine when refill() or push() can be called
    /// without blocking.
    pub fn poll_fd(&mut self) -> Result<c_int> {
        let ret = unsafe { ffi::iio_buffer_get_poll_fd(self.buf) };
        sys_result(i32::from(ret), ret)
    }

    /// Make the push or refill calls blocking or not.
    pub fn set_blocking_mode(&mut self, blocking: bool) -> Result<()> {
        let ret = unsafe { ffi::iio_buffer_set_blocking_mode(self, blocking.buf) };
        sys_result(ret, ())
    }

    /// Fetch more samples from the hardware
    ///
    /// This is only valid for input buffers
    pub fn refill(&mut self) -> Result<usize> {
        let ret = unsafe { ffi::iio_buffer_refill(self.buf) };
        sys_result(ret as i32, ret as usize)
    }

    /// Send the samples to the hardware.
    ///
    /// This is only valid for output buffers
    pub fn push(&mut self) -> Result<usize> {
        let ret = unsafe { ffi::iio_buffer_push(self.buf) };
        sys_result(ret as i32, ret as usize)
    }

    /// Send a given number of samples to the hardware.
    ///
    /// `n` The number of samples to send
    /// This is only valid for output buffers
    pub fn push_partial(&mut self, n: usize) -> Result<usize> {
        let ret = unsafe { ffi::iio_buffer_push_partial(self.buf, n) };
        sys_result(ret as i32, ret as usize)
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

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { ffi::iio_buffer_destroy(self.buf) }
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


