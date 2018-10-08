// libiio-sys/src/channel.rs
//
//!
//!

use ffi;

pub struct Channel {
    chan: *mut ffi::iio_channel,
}

