// libiio-sys/src/channel.rs
//
//!
//!

use ffi;
use super::*;

#[derive(Debug, PartialEq)]
pub enum ChannelType {
    Input,
    Output
}

/// An Industrial I/O Device Channel
pub struct Channel {
    pub(crate) chan: *mut ffi::iio_channel,
}

impl Channel {
    /// Retrieves the name of the channel (e.g. <b><i>vccint</i></b>)
    pub fn name(&self) -> Option<String> {
        let pstr = unsafe { ffi::iio_channel_get_name(self.chan) };
        cstring_opt(pstr)
    }

    /// @brief Retrieve the channel ID (e.g. <b><i>voltage0</i></b>)
    pub fn id(&self) -> Option<String> {
        let pstr = unsafe { ffi::iio_channel_get_id(self.chan) };
        cstring_opt(pstr)
    }

    /// Determines if the channel is a scan element
    ///
    /// A scan element is a channel that can generate samples (for an
    /// input  channel) or receive samples (for an output channel) after
    /// being enabled.
    pub fn is_scan_element(&self) -> bool {
        unsafe { ffi::iio_channel_is_scan_element(self.chan) }
    }

    /// Enable the channel
    ///
    /// Before creating a buffer, at least one channel of the device
    /// must be enabled.
    pub fn enable(&mut self) {
        unsafe { ffi::iio_channel_enable(self.chan) };
    }

    /// Disables the channel
    pub fn disable(&mut self) {
        unsafe { ffi::iio_channel_disable(self.chan) };
    }

    /// Determines if the channel is enabled
    pub fn is_enabled(&self) -> bool {
        unsafe { ffi::iio_channel_is_enabled(self.chan) }
    }

}



