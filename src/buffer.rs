// libiio-sys/src/buffer.rs
//
//!
//!

use ffi;

pub struct Buffer {
    buf: *mut ffi::iio_buffer,
}

