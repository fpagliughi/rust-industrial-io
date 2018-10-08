// libiio-sys/src/device.rs
//
//!
//!

use ffi;

pub struct Device {
    dev: *mut ffi::iio_device,
}


