// libiio-sys/src/buffer.rs
//
// Copyright (c) 2018-2021, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//! Industrial I/O Buffers.
//!
//! The process of capturing samples from or uploading samples to hardware is
//! managed using [`Buffer`] and related methods.
//!
//! It is important to keep in mind that an instance of [`Buffer`] is always
//! coupled to exactly **one instance of [`Device`]**, and vice-versa.
//! [`Buffer`]s are allocated on a per-[`Device`] basis, and not per
//! [`Channel`]. In order to control which [`Channel`]s to capture in a
//! [`Buffer`], the respective [`Channel`]s must be [enabled][enable_chan] or
//! [disabled][disable_chan].
//!
//! The very first step when working with [`Buffer`]s is to
//! [enable][enable_chan] the capture [`Channel`]s that we want to use, and
//! [disable][disable_chan] those that we don't need. This is done with the
//! functions [`Channel::enable()`] and [`Channel::disable()`]. Note that the
//! [`Channel`]s will really be enabled or disabled when the [`Buffer`]-object
//! is created.
//!
//! Also, not all [`Channel`]s can be enabled. To know whether or not one
//! [`Channel`] can be enabled, use [`Channel::is_scan_element()`].
//!
//! Once the [`Channel`]s have been enabled, and [triggers assigned] (for
//! triggered [`Buffer`]s) the [`Buffer`] object can be created from the
//! [`Device`] object that will be used, with the function
//! [`Device::create_buffer()`]. This call will fail if no [`Channel`]s have
//! been enabled, or for triggered buffers, if the trigger has not been
//! assigned.
//!
//! [`Buffer`] objects are automatically dropped when their scope ends.
//!
//! For additional information on actually working with [`Buffer`]s, including
//! some examples, refer to the [`Buffer` documentation][`Buffer`].
//!
//! Most parts of the documentation for this module were taken from the [libiio
//! documentation](https://analogdevicesinc.github.io/libiio/master/libiio/index.html)
//!
//! [enable_chan]: crate::channel::Channel::enable()
//! [disable_chan]: crate::channel::Channel::disable()
//! [triggers assigned]: crate::device::Device::set_trigger()
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(missing_docs)]

use std::{
    collections::HashMap,
    marker::PhantomData,
    mem,
    os::raw::{c_int, c_longlong, c_void},
    ptr,
};

use super::*;
use crate::ffi;

/// An Industrial I/O input or output buffer.
///
/// See [here][crate::buffer] for a detailed explanation of how buffers work.
///
/// # Examples
///
#[derive(Debug)]
pub struct Buffer {
    /// The underlying buffer from the C library
    pub(crate) buf: *mut ffi::iio_buffer,
    /// The buffer capacity (# samples from each channel)
    pub(crate) cap: usize,
    /// Copy of the device to which this device is attached.
    pub(crate) dev: Device,
}

impl Buffer {
    /// Get the buffer size.
    ///
    /// Get the buffer capacity in number of samples from each channel that
    /// the buffer can hold.
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Gets a reference to the device to which this buffer is attached.
    pub fn device(&self) -> &Device {
        &self.dev
    }

    /// Gets a pollable file descriptor for the buffer.
    ///
    /// This can be used to determine when [`Buffer::refill()`] or
    /// [`Buffer::push()`] can be called without blocking.
    pub fn poll_fd(&self) -> Result<c_int> {
        let ret = unsafe { ffi::iio_buffer_get_poll_fd(self.buf) };
        sys_result(i32::from(ret), ret)
    }

    /// Make calls to [`Buffer::push()`] or [`Buffer::refill()`] blocking or not.
    ///
    /// A [`Device`] is blocking by default.
    pub fn set_blocking_mode(&self, blocking: bool) -> Result<()> {
        let ret = unsafe { ffi::iio_buffer_set_blocking_mode(self.buf, blocking) };
        sys_result(ret, ())
    }

    /// Fetch more samples from the hardware.
    ///
    /// This is only valid for input buffers.
    pub fn refill(&mut self) -> Result<usize> {
        let ret = unsafe { ffi::iio_buffer_refill(self.buf) };
        sys_result(ret as i32, ret as usize)
    }

    /// Send the samples to the hardware.
    ///
    /// This is only valid for output buffers.
    pub fn push(&self) -> Result<usize> {
        let ret = unsafe { ffi::iio_buffer_push(self.buf) };
        sys_result(ret as i32, ret as usize)
    }

    /// Send a given number of samples to the hardware.
    ///
    /// This is only valid for output buffers. Note that the number of samples
    /// explicitly doesn't refer to their size in bytes, but the actual number
    /// of samples, regardless of the sample size in memory.
    pub fn push_partial(&self, num_samples: usize) -> Result<usize> {
        let ret = unsafe { ffi::iio_buffer_push_partial(self.buf, num_samples) };
        sys_result(ret as i32, ret as usize)
    }

    /// Cancel all buffer operations.
    ///
    /// This function cancels all outstanding [`Buffer`] operations previously
    /// scheduled. This means any pending [`Buffer::push()`] or
    /// [`Buffer::refill()`] operation will abort and return immediately, any
    /// further invocations of these functions on the same buffer will return
    /// immediately with an error.
    ///
    /// Usually [`Buffer::push()`] and [`Buffer::refill()`] will block until
    /// either all data has been transferred or a timeout occurs. This can,
    /// depending on the configuration, take a significant amount of time.
    /// [`Buffer::cancel()`] is useful to bypass these conditions if the
    /// [`Buffer`] operation is supposed to be stopped in response to an
    /// external event (e.g. user input).
    ///
    /// TODO: @fpagliughi, is this true for the rust binding, too?
    /// To be able to capture additional data after calling this function the
    /// buffer should be destroyed and then re-created.
    ///
    /// This function can be called multiple times for the same buffer, but all
    /// but the first invocation will be without additional effect.
    pub fn cancel(&self) {
        unsafe {
            ffi::iio_buffer_cancel(self.buf);
        }
    }

    /// Determines if the device has any buffer-specific attributes
    pub fn has_attrs(&self) -> bool {
        unsafe { ffi::iio_device_get_buffer_attrs_count(self.dev.dev) > 0 }
    }

    /// Gets the number of buffer-specific attributes
    pub fn num_attrs(&self) -> usize {
        unsafe { ffi::iio_device_get_buffer_attrs_count(self.dev.dev) as usize }
    }

    /// Gets the name of the buffer-specific attribute at the index
    pub fn get_attr(&self, idx: usize) -> Result<String> {
        let pstr = unsafe { ffi::iio_device_get_buffer_attr(self.dev.dev, idx as c_uint) };
        cstring_opt(pstr).ok_or(Error::InvalidIndex)
    }

    /// Try to find a buffer-specific attribute by its name
    pub fn find_attr(&self, name: &str) -> Option<String> {
        let cname = cstring_or_bail!(name);
        let pstr = unsafe { ffi::iio_device_find_buffer_attr(self.dev.dev, cname.as_ptr()) };
        cstring_opt(pstr)
    }

    /// Determines if a buffer-specific attribute exists
    pub fn has_attr(&self, name: &str) -> bool {
        let cname = cstring_or_bail_false!(name);
        let pstr = unsafe { ffi::iio_device_find_buffer_attr(self.dev.dev, cname.as_ptr()) };
        !pstr.is_null()
    }

    /// Reads a buffer-specific attribute
    ///
    /// `attr` The name of the attribute
    pub fn attr_read<T: FromAttribute>(&self, attr: &str) -> Result<T> {
        let sval = self.attr_read_str(attr)?;
        T::from_attr(&sval)
    }

    /// Reads a buffer-specific attribute as a string
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_str(&self, attr: &str) -> Result<String> {
        let mut buf = vec![0 as c_char; ATTR_BUF_SIZE];
        let attr = CString::new(attr)?;
        let ret = unsafe {
            ffi::iio_device_buffer_attr_read(
                self.dev.dev,
                attr.as_ptr(),
                buf.as_mut_ptr(),
                buf.len(),
            )
        };
        sys_result(ret as i32, ())?;
        let s = unsafe {
            CStr::from_ptr(buf.as_ptr())
                .to_str()
                .map_err(|_| Error::StringConversionError)?
        };
        Ok(s.into())
    }

    /// Reads a buffer-specific attribute as a boolean
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_bool(&self, attr: &str) -> Result<bool> {
        let mut val: bool = false;
        let attr = CString::new(attr)?;
        let ret =
            unsafe { ffi::iio_device_buffer_attr_read_bool(self.dev.dev, attr.as_ptr(), &mut val) };
        sys_result(ret, val)
    }

    /// Reads a buffer-specific attribute as an integer (i64)
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_int(&self, attr: &str) -> Result<i64> {
        let mut val: c_longlong = 0;
        let attr = CString::new(attr)?;
        let ret = unsafe {
            ffi::iio_device_buffer_attr_read_longlong(self.dev.dev, attr.as_ptr(), &mut val)
        };
        sys_result(ret, val as i64)
    }

    /// Reads a buffer-specific attribute as a floating-point (f64) number
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_float(&self, attr: &str) -> Result<f64> {
        let mut val: f64 = 0.0;
        let attr = CString::new(attr)?;
        let ret = unsafe {
            ffi::iio_device_buffer_attr_read_double(self.dev.dev, attr.as_ptr(), &mut val)
        };
        sys_result(ret, val)
    }

    /// Reads all the buffer-specific attributes.
    /// This is especially useful when using the network backend to
    /// retrieve all the attributes with a single call.
    pub fn attr_read_all(&self) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        let pmap = &mut map as *mut _ as *mut c_void;
        let ret = unsafe {
            ffi::iio_device_buffer_attr_read_all(self.dev.dev, Some(attr_read_all_cb), pmap)
        };
        sys_result(ret, map)
    }

    /// Writes a buffer-specific attribute
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write<T: ToAttribute>(&self, attr: &str, val: T) -> Result<()> {
        let sval = T::to_attr(&val)?;
        self.attr_write_str(attr, &sval)
    }

    /// Writes a buffer-specific attribute as a string
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_str(&self, attr: &str, val: &str) -> Result<()> {
        let attr = CString::new(attr)?;
        let sval = CString::new(val)?;
        let ret = unsafe {
            ffi::iio_device_buffer_attr_write(self.dev.dev, attr.as_ptr(), sval.as_ptr())
        };
        sys_result(ret as i32, ())
    }

    /// Writes a buffer-specific attribute as a boolean
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_bool(&self, attr: &str, val: bool) -> Result<()> {
        let attr = CString::new(attr)?;
        let ret =
            unsafe { ffi::iio_device_buffer_attr_write_bool(self.dev.dev, attr.as_ptr(), val) };
        sys_result(ret, ())
    }

    /// Writes a buffer-specific attribute as an integer (i64)
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_int(&self, attr: &str, val: i64) -> Result<()> {
        let attr = CString::new(attr)?;
        let ret =
            unsafe { ffi::iio_device_buffer_attr_write_longlong(self.dev.dev, attr.as_ptr(), val) };
        sys_result(ret, ())
    }

    /// Writes a buffer-specific attribute as a floating-point (f64) number
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_float(&self, attr: &str, val: f64) -> Result<()> {
        let attr = CString::new(attr)?;
        let ret =
            unsafe { ffi::iio_device_buffer_attr_write_double(self.dev.dev, attr.as_ptr(), val) };
        sys_result(ret, ())
    }

    /// Gets an iterator for the buffer attributes in the device
    pub fn attributes(&self) -> AttrIterator {
        AttrIterator { buf: self, idx: 0 }
    }

    /// Set the number of kernel buffers for the device.
    pub fn set_num_kernel_buffers(&self, n: u32) -> Result<()> {
        let ret = unsafe { ffi::iio_device_set_kernel_buffers_count(self.dev.dev, n as c_uint) };
        sys_result(ret, ())
    }

    /// Gets an iterator for the data from a channel.
    pub fn channel_iter<T>(&self, chan: &Channel) -> IntoIter<T> {
        unsafe {
            let begin = ffi::iio_buffer_first(self.buf, chan.chan) as *mut T;
            let end = ffi::iio_buffer_end(self.buf) as *const T;
            let ptr = begin;
            let step: isize = ffi::iio_buffer_step(self.buf) / mem::size_of::<T>() as isize;

            IntoIter {
                phantom: PhantomData,
                ptr,
                end,
                step,
            }
        }
    }
}

/// Destroy the underlying buffer when the object scope ends.
impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { ffi::iio_buffer_destroy(self.buf) }
    }
}

/// An iterator that moves channel data out of a buffer.
#[derive(Debug)]
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

/// Iterator over the buffer attributes
/// 'a Lifetime of the Buffer
#[derive(Debug)]
pub struct AttrIterator<'a> {
    /// Reference to the Buffer that we're scanning for attributes
    buf: &'a Buffer,
    /// Index to the next Buffer attribute from the iterator
    idx: usize,
}

impl<'a> Iterator for AttrIterator<'a> {
    type Item = String;

    /// Gets the next Buffer attribute from the iterator
    fn next(&mut self) -> Option<Self::Item> {
        match self.buf.get_attr(self.idx) {
            Ok(name) => {
                self.idx += 1;
                Some(name)
            }
            Err(_) => None,
        }
    }
}
