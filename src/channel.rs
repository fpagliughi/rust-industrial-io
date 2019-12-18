// libiio-sys/src/channel.rs
//
// Copyright (c) 2018-2019, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//! Industrial I/O Channels
//!

use std::{mem, str};
use std::ffi::CString;
use std::os::raw::{c_void, c_int, c_uint, c_longlong};
use std::collections::HashMap;

use ffi;
use super::*;

/// The type of data associated with a channel.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelType {
    Voltage = ffi::iio_chan_type_IIO_VOLTAGE,
    Current = ffi::iio_chan_type_IIO_CURRENT,
    Power = ffi::iio_chan_type_IIO_POWER,
    Accel = ffi::iio_chan_type_IIO_ACCEL,
    AnglVel = ffi::iio_chan_type_IIO_ANGL_VEL,
    Magn = ffi::iio_chan_type_IIO_MAGN,
    Ligtht = ffi::iio_chan_type_IIO_LIGHT,
    Intensity = ffi::iio_chan_type_IIO_INTENSITY,
    Proximity = ffi::iio_chan_type_IIO_PROXIMITY,
    Temp = ffi::iio_chan_type_IIO_TEMP,
    Incli = ffi::iio_chan_type_IIO_INCLI,
    Rot = ffi::iio_chan_type_IIO_ROT,
    Angl = ffi::iio_chan_type_IIO_ANGL,
    Timestamp = ffi::iio_chan_type_IIO_TIMESTAMP,
    Capacitance = ffi::iio_chan_type_IIO_CAPACITANCE,
    AltVoltage = ffi::iio_chan_type_IIO_ALTVOLTAGE,
    Cct = ffi::iio_chan_type_IIO_CCT,
    Pressure = ffi::iio_chan_type_IIO_PRESSURE,
    HumidityRelative = ffi::iio_chan_type_IIO_HUMIDITYRELATIVE,
    Activity = ffi::iio_chan_type_IIO_ACTIVITY,
    Steps = ffi::iio_chan_type_IIO_STEPS,
    Energy = ffi::iio_chan_type_IIO_ENERGY,
    Distance = ffi::iio_chan_type_IIO_DISTANCE,
    Velocity = ffi::iio_chan_type_IIO_VELOCITY,
    Concentration = ffi::iio_chan_type_IIO_CONCENTRATION,
    Resistance = ffi::iio_chan_type_IIO_RESISTANCE,
    Ph = ffi::iio_chan_type_IIO_PH,
    UvIndex = ffi::iio_chan_type_IIO_UVINDEX,
    ElectricalConductivity = ffi::iio_chan_type_IIO_ELECTRICALCONDUCTIVITY,
    Count = ffi::iio_chan_type_IIO_COUNT,
    Index = ffi::iio_chan_type_IIO_INDEX,
    Gravity = ffi::iio_chan_type_IIO_GRAVITY,
    Unknown = ffi::iio_chan_type_IIO_CHAN_TYPE_UNKNOWN,
}

/// The format of a data sample.
#[derive(Debug, Copy, Clone)]
pub struct DataFormat {
    /// The data format struct from the C library
    data_fmt: ffi::iio_data_format,
}

impl DataFormat {
    fn new(data_fmt: ffi::iio_data_format) -> Self {
        DataFormat { data_fmt, }
    }

    /// Gets total length of the sample, in bits.
    pub fn length(&self) -> u32 {
        u32::from(self.data_fmt.length)
    }

    /// Gets the length of valid data in the sample, in bits.
    pub fn bits(&self) -> u32 {
        u32::from(self.data_fmt.bits)
    }

    /// Right-shift to apply when converting sample.
    pub fn shift(&self) -> u32 {
        u32::from(self.data_fmt.shift)
    }

    /// Determines if the sample is signed
    pub fn is_signed(&self) -> bool {
        self.data_fmt.is_signed
    }

    /// Determines if the sample is fully defined, sign extended, etc.
    pub fn is_fully_defined(&self) -> bool {
        self.data_fmt.is_fully_defined
    }

    /// Determines if the sample is in big-endian format
    pub fn is_big_endian(&self) -> bool {
        self.data_fmt.is_be
    }

    /// Determinesif the sample should be scaled when converted
    pub fn with_scale(&self) -> bool {
        self.data_fmt.with_scale
    }

    /// Contains the scale to apply if with_scale is set
    pub fn scale(&self) -> f64 {
        self.data_fmt.scale
    }

    /// Number of times length repeats
    pub fn repeat(&self) -> u32 {
        u32::from(self.data_fmt.repeat)
    }
}

/// An Industrial I/O Device Channel
pub struct Channel {
    pub(crate) chan: *mut ffi::iio_channel,
    #[allow(dead_code)]  // looks like it's unused, but really it's holding the Device's lifetime for libiio safety.
    pub(crate) ctx: Context,
}

impl Channel {
    /// Retrieves the name of the channel (e.g. <b><i>vccint</i></b>)
    pub fn name(&self) -> Option<String> {
        let pstr = unsafe { ffi::iio_channel_get_name(self.chan) };
        cstring_opt(pstr)
    }

    /// Retrieve the channel ID (e.g. <b><i>voltage0</i></b>)
    pub fn id(&self) -> Option<String> {
        let pstr = unsafe { ffi::iio_channel_get_id(self.chan) };
        cstring_opt(pstr)
    }

    /// Determines if this is an output channel.
    pub fn is_output(&self) -> bool {
        unsafe { ffi::iio_channel_is_output(self.chan) }
    }

    /// Determines if the channel is a scan element
    ///
    /// A scan element is a channel that can generate samples (for an
    /// input  channel) or receive samples (for an output channel) after
    /// being enabled.
    pub fn is_scan_element(&self) -> bool {
        unsafe { ffi::iio_channel_is_scan_element(self.chan) }
    }

    pub fn index(&self) -> Result<usize> {
        let ret = unsafe { ffi::iio_channel_get_index(self.chan) };
        sys_result(ret as i32, ret as usize)
    }

    /// Gets the number of context-specific attributes
    pub fn num_attrs(&self) -> usize {
        let n = unsafe { ffi::iio_channel_get_attrs_count(self.chan) };
        n as usize
    }

    /// Determines if the channel has the specified attribute.
    pub fn has_attr(&self, attr: &str) -> bool {
        let attr = match CString::new(attr) {
            Ok(s) => s,
            Err(_) => return false,
        };
        unsafe { !ffi::iio_channel_find_attr(self.chan, attr.as_ptr()).is_null() }
    }

    /// Gets the channel-specific attribute at the index
    pub fn get_attr(&self, idx: usize) -> Result<String> {
        let pstr = unsafe { ffi::iio_channel_get_attr(self.chan, idx as c_uint) };
        cstring_opt(pstr).ok_or_else(|| "Invalid index".into())
    }

    /// Try to find the channel-specific attribute by name.
    pub fn find_attr(&self, attr: &str) -> Option<String> {
        let attr = match CString::new(attr) {
            Ok(s) => s,
            Err(_) => return None,
        };
        let pstr = unsafe { ffi::iio_channel_find_attr(self.chan, attr.as_ptr()) };
        cstring_opt(pstr)
    }

    /// Reads a channel-specific attribute as a boolean
    /// `attr` The name of the attribute
    pub fn attr_read_bool(&self, attr: &str) -> Result<bool> {
        let mut val: bool = false;
        let attr = CString::new(attr)?;
        let ret = unsafe {
            ffi::iio_channel_attr_read_bool(self.chan, attr.as_ptr(), &mut val)
        };
        sys_result(ret, val)
    }

    /// Reads a channel-specific attribute as an integer (i64)
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_int(&self, attr: &str) -> Result<i64> {
        let mut val: c_longlong = 0;
        let attr = CString::new(attr)?;
        let ret = unsafe {
            ffi::iio_channel_attr_read_longlong(self.chan, attr.as_ptr(), &mut val)
        };
        sys_result(ret, val as i64)
    }

    /// Reads a channel-specific attribute as a floating-point (f64) number
    ///
    /// `attr` The name of the attribute
    pub fn attr_read_float(&self, attr: &str) -> Result<f64> {
        let mut val: f64 = 0.0;
        let attr = CString::new(attr)?;
        let ret = unsafe {
            ffi::iio_channel_attr_read_double(self.chan, attr.as_ptr(), &mut val)
        };
        sys_result(ret, val)
    }

    // Callback from the C lib to extract the collection of all
    // channel-specific attributes. See attr_read_all().
    unsafe extern "C" fn attr_read_all_cb(_chan: *mut ffi::iio_channel,
                                          attr: *const c_char,
                                          val: *const c_char, _len: usize,
                                          pmap: *mut c_void) -> c_int {
        if attr.is_null() || val.is_null() || pmap.is_null() {
            return -1;
        }

        let attr = CStr::from_ptr(attr).to_string_lossy().to_string();
        // TODO: We could/should check val[len-1] == '\x0'
        let val = CStr::from_ptr(val).to_string_lossy().to_string();
        let map: &mut HashMap<String,String> = &mut *(pmap as *mut _);
        map.insert(attr, val);
        0
    }

    /// Reads all the channel-specific attributes.
    /// This is especially useful when using the network backend to
    /// retrieve all the attributes with a single call.
    pub fn attr_read_all(&self) -> Result<HashMap<String,String>> {
        let mut map = HashMap::new();
        let pmap = &mut map as *mut _ as *mut c_void;
        let ret = unsafe {
            ffi::iio_channel_attr_read_all(self.chan, Some(Channel::attr_read_all_cb), pmap)
        };
        sys_result(ret, map)
    }

    /// Writes a channel-specific attribute as a boolean
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_bool(&self, attr: &str, val: bool) -> Result<()> {
        let attr = CString::new(attr)?;
        let ret = unsafe {
            ffi::iio_channel_attr_write_bool(self.chan, attr.as_ptr(), val)
        };
        sys_result(ret, ())
    }

    /// Writes a channel-specific attribute as an integer (i64)
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_int(&self, attr: &str, val: i64) -> Result<()> {
        let attr = CString::new(attr)?;
        let ret = unsafe {
            ffi::iio_channel_attr_write_longlong(self.chan, attr.as_ptr(), val)
        };
        sys_result(ret, ())
    }

    /// Writes a channel-specific attribute as a floating-point (f64) number
    ///
    /// `attr` The name of the attribute
    /// `val` The value to write
    pub fn attr_write_float(&self, attr: &str, val: f64) -> Result<()> {
        let attr = CString::new(attr)?;
        let ret = unsafe {
            ffi::iio_channel_attr_write_double(self.chan, attr.as_ptr(), val)
        };
        sys_result(ret, ())
    }

    /// Gets an iterator for the attributes of the channel
    pub fn attrs(&self) -> AttrIterator {
        AttrIterator {
            chan: self,
            idx: 0
        }
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

    /// Gets the data format for the channel
    pub fn data_format(&self) -> DataFormat {
        unsafe {
            let pfmt = ffi::iio_channel_get_data_format(self.chan);
            DataFormat::new(*pfmt)
        }
    }

    /// Gets the type of data associated with the channel
    pub fn channel_type(&self) -> ChannelType {
        // TODO: We're trusting that the lib returns a valid enum.
        unsafe {
            let n = ffi::iio_channel_get_type(self.chan);
            mem::transmute(n)
        }
    }
}

pub struct AttrIterator<'a> {
    chan: &'a Channel,
    idx: usize,
}

impl<'a> Iterator for AttrIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.chan.get_attr(self.idx) {
            Ok(name) => {
                self.idx += 1;
                Some(name)
            },
            Err(_) => None
        }
    }
}

