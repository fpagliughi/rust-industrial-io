// libiio-sys/src/context.rs
//
// Copyright (c) 2018-2025, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//! Industrial I/O Contexts.
//!

use crate::{cstring_opt, ffi, sys_result, Device, Error, Result, Version};
use nix::errno::Errno;
use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_uint},
    ptr, slice, str,
    sync::Arc,
    time::Duration,
};

/////////////////////////////////////////////////////////////////////////////

/// An Industrial I/O Context
///
/// This object maintains a reference counted pointer to the context object
/// of the underlying library's `iio_context` object. Once all references to
/// the Context object have been dropped, the underlying `iio_context` will be
/// destroyed. This is done to make creation and use of a single Device more
/// ergonomic by removing the need to manage the lifetime of the Context.
#[derive(Debug, Clone)]
pub struct Context {
    inner: Arc<InnerContext>,
}

/// Backends for I/O Contexts.
///
/// An I/O [`Context`] relies on a backend that provides sensor data.
#[derive(Debug)]
pub enum Backend<'a> {
    /// Use the default backend. This will create a network context if the
    /// IIOD_REMOTE environment variable is set to the hostname where the
    /// IIOD server runs. If set to an empty string, the server will be
    /// discovered using ZeroConf. If the environment variable is not set,
    /// a local context will be created instead.
    Default,
    /// XML Backend, creates a Context from an XML file. Here the string is
    /// the name of the file.
    /// Example Parameter:
    /// "/home/user/file.xml"
    Xml(&'a str),
    /// XML Backend, creates a Context from an in-memory XML string. Here
    /// the string _is_ the XML description.
    XmlMem(&'a str),
    /// Network Backend, creates a Context through a network connection.
    /// Requires a hostname, IPv4 or IPv6 address to connect to another host
    /// that is running the [IIO Daemon]. If an empty string is provided,
    /// automatic discovery through ZeroConf is performed (if available in IIO).
    /// Example Parameter:
    ///
    /// - "192.168.2.1" to connect to given IPv4 host, **or**
    /// - "localhost" to connect to localhost running IIOD, **or**
    /// - "plutosdr.local" to connect to host with given hostname, **or**
    /// - "" for automatic discovery
    ///
    /// [IIO Daemon]: https://github.com/analogdevicesinc/libiio/tree/master/iiod
    Network(&'a str),
    /// USB Backend, creates a context through a USB connection.
    /// If only a single USB device is attached, provide an empty String ("")
    /// to use that. When more than one usb device is attached, requires bus,
    /// address, and interface parts separated with a dot.
    /// Example Parameter: "3.32.5"
    Usb(&'a str),
    /// Serial Backend, creates a context through a serial connection.
    /// Requires (Values in parentheses show examples):
    ///
    /// - a port (/dev/ttyUSB0),
    /// - baud_rate (default 115200)
    /// - serial port configuration
    ///     - data bits (5 6 7 8 9)
    ///     - parity ('n' none, 'o' odd, 'e' even, 'm' mark, 's' space)
    ///     - stop bits (1 2)
    ///     - flow control ('\0' none, 'x' Xon Xoff, 'r' RTSCTS, 'd' DTRDSR)
    ///
    /// Example Parameters:
    ///
    /// - "/dev/ttyUSB0,115200", **or**
    /// - "/dev/ttyUSB0,115200,8n1"
    Serial(&'a str),
    /// "Guess" the backend to use from the URI that's supplied. This merely
    /// provides compatibility with [`iio_create_context_from_uri`] from the
    /// underlying IIO C-library. Refer to the IIO docs for information on how
    /// to format this parameter.
    ///
    /// [`iio_create_context_from_uri`]: https://analogdevicesinc.github.io/libiio/master/libiio/group__Context.html#gafdcee40508700fa395370b6c636e16fe
    Uri(&'a str),
    /// Local Backend, only available on Linux hosts. Sensors to work with are
    /// part of the system and accessible in sysfs (under `/sys/...`).
    #[cfg(target_os = "linux")]
    Local,
}

/// This holds a pointer to the library context.
/// When it is dropped, the library context is destroyed.
#[derive(Debug)]
pub struct InnerContext {
    /// Pointer to a libiio Context object
    pub(crate) ctx: *mut ffi::iio_context,
}

impl InnerContext {
    /// Tries to create the inner context from a C context pointer.
    ///
    /// This should be called _right after_ creating the C context as it
    /// will use the last error on failure.
    fn new(ctx: *mut ffi::iio_context) -> Result<Self> {
        if ctx.is_null() {
            Err(Error::from(Errno::last()))
        }
        else {
            Ok(Self { ctx })
        }
    }

    /// Tries to create a full, deep, copy of the underlying context.
    ///
    /// This creates a full copy of the actual context held in the underlying
    /// C library. This is useful if you want to give a separate copy to each
    /// thread in an application, which could help performance.
    pub fn try_clone(&self) -> Result<Self> {
        Self::new(unsafe { ffi::iio_context_clone(self.ctx) })
    }
}

impl Drop for InnerContext {
    /// Dropping destroys the underlying C context.
    ///
    /// When held by [`Context`] references, this should happen when the last
    /// context referring to it goes out of scope.
    fn drop(&mut self) {
        unsafe { ffi::iio_context_destroy(self.ctx) };
    }
}

// The inner context can be sent to another thread.
unsafe impl Send for InnerContext {}

// The inner context can be shared with another thread.
unsafe impl Sync for InnerContext {}

impl Context {
    /// Creates a default context from a local or remote IIO device.
    ///
    /// # Notes
    ///
    /// This will create a network context if the `IIOD_REMOTE`
    /// environment variable is set to the hostname where the IIOD server
    /// runs. If set to an empty string, the server will be discovered using
    /// `ZeroConf`. If the environment variable is not set, a local context
    /// will be created instead.
    pub fn new() -> Result<Self> {
        Self::from_ptr(unsafe { ffi::iio_create_default_context() })
    }

    /// Create an IIO Context.
    ///
    /// A context contains one or more devices (i.e. sensors) that can provide
    /// data. Note that any device can always only be associated with one
    /// context!
    ///
    /// Contexts rely on [`Backend`]s to discover available sensors. Multiple
    /// [`Backend`]s are supported.
    ///
    /// # Examples
    ///
    /// Create a context to work with sensors on the local system
    /// (Only supported for Linux hosts):
    ///
    /// ```no_run
    /// use industrial_io as iio;
    ///
    /// let ctx = iio::Context::with_backend(iio::Backend::Local);
    /// ```
    ///
    /// Create a context that works with senors on some network host:
    ///
    /// ```no_run
    /// use industrial_io as iio;
    ///
    /// let ctx_ip = iio::Context::with_backend(iio::Backend::Network("192.168.2.1"));
    /// let ctx_host = iio::Context::with_backend(iio::Backend::Network("runs-iiod.local"));
    /// let ctx_auto = iio::Context::with_backend(iio::Backend::Network(""));
    /// ```
    ///
    /// Creates a Context using some arbitrary URI (like it is used in the
    /// underlying IIO C-library):
    ///
    /// ```no_run
    /// use industrial_io as iio;
    ///
    /// let ctx = iio::Context::with_backend(iio::Backend::Uri("ip:192.168.2.1"));
    /// ```
    pub fn with_backend(be: Backend) -> Result<Self> {
        Self::from_ptr(unsafe {
            match be {
                Backend::Default => ffi::iio_create_default_context(),
                Backend::Xml(name) => {
                    let name = CString::new(name)?;
                    ffi::iio_create_xml_context(name.as_ptr())
                }
                Backend::XmlMem(xml) => {
                    let n = xml.len();
                    let xml = CString::new(xml)?;
                    ffi::iio_create_xml_context_mem(xml.as_ptr(), n)
                }
                Backend::Network(host) => {
                    let host = CString::new(host)?;
                    ffi::iio_create_network_context(host.as_ptr())
                }
                Backend::Usb(device) => {
                    let uri = CString::new(format!("usb:{}", device))?;
                    ffi::iio_create_context_from_uri(uri.as_ptr())
                }
                Backend::Serial(tty) => {
                    let uri = CString::new(format!("serial:{}", tty))?;
                    ffi::iio_create_context_from_uri(uri.as_ptr())
                }
                Backend::Uri(uri) => {
                    let uri = CString::new(uri)?;
                    ffi::iio_create_context_from_uri(uri.as_ptr())
                }
                #[cfg(target_os = "linux")]
                Backend::Local => ffi::iio_create_local_context(),
            }
        })
    }

    /// Creates a context specified by the `uri`.
    pub fn from_uri(uri: &str) -> Result<Self> {
        Self::with_backend(Backend::Uri(uri))
    }

    /// Creates a network backend on the specified host.
    ///
    /// This is a convenience function to create a context with the network
    /// back end.
    pub fn from_network(hostname: &str) -> Result<Self> {
        Self::with_backend(Backend::Network(hostname))
    }

    /// Creates a context from an existing "inner" object.
    pub fn from_inner(inner: InnerContext) -> Self {
        Self::from(inner)
    }

    /// Creates a Rust Context object from a C context pointer.
    fn from_ptr(ctx: *mut ffi::iio_context) -> Result<Self> {
        let inner = InnerContext::new(ctx)?;
        Ok(Self::from_inner(inner))
    }

    /// Try to create a clone of the inner underlying context.
    ///
    /// The inner context wraps the C library context. Cloning it makes
    /// a full copy of the C context.
    pub fn try_clone_inner(&self) -> Result<InnerContext> {
        self.inner.try_clone()
    }

    /// Tries to release the inner context.
    ///
    /// This attempts to release and return the [`InnerContext`], which
    /// succeeds if this is the only [`Context`] referring to it. If there are
    /// other references, an error is returned with a [`Context`].
    pub fn try_release_inner(self) -> std::result::Result<InnerContext, Self> {
        match Arc::try_unwrap(self.inner) {
            Ok(inner) => Ok(inner),
            Err(inner_ptr) => Err(Self { inner: inner_ptr }),
        }
    }

    /// Make a new context based on a full copy of underlying C context.
    pub fn try_deep_clone(&self) -> Result<Self> {
        let inner = self.inner.try_clone()?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Get the name of the context.
    /// This should be "local", "xml", or "network" depending on how the context was created.
    pub fn name(&self) -> String {
        let pstr = unsafe { ffi::iio_context_get_name(self.inner.ctx) };
        cstring_opt(pstr).unwrap_or_default()
    }

    /// Get a description of the context
    pub fn description(&self) -> String {
        let pstr = unsafe { ffi::iio_context_get_description(self.inner.ctx) };
        cstring_opt(pstr).unwrap_or_default()
    }

    /// Get the version of the backend in use
    pub fn version(&self) -> Version {
        let mut major: c_uint = 0;
        let mut minor: c_uint = 0;

        const BUF_SZ: usize = 8;
        let mut buf = vec![b' ' as c_char; BUF_SZ];
        let pbuf = buf.as_mut_ptr();

        unsafe { ffi::iio_context_get_version(self.inner.ctx, &mut major, &mut minor, pbuf) };

        let sgit = unsafe {
            if buf.contains(&0) {
                CStr::from_ptr(pbuf).to_owned()
            }
            else {
                let slc = str::from_utf8(slice::from_raw_parts(pbuf.cast(), BUF_SZ)).unwrap();
                CString::new(slc).unwrap()
            }
        };
        Version {
            major: major as u32,
            minor: minor as u32,
            git_tag: sgit.to_string_lossy().into_owned(),
        }
    }

    /// Obtain the XML representation of the context.
    pub fn xml(&self) -> String {
        let pstr = unsafe { ffi::iio_context_get_xml(self.inner.ctx) };
        cstring_opt(pstr).unwrap_or_default()
    }

    /// Determines if the context has any attributes
    pub fn has_attrs(&self) -> bool {
        unsafe { ffi::iio_context_get_attrs_count(self.inner.ctx) > 0 }
    }

    /// Gets the number of context-specific attributes
    pub fn num_attrs(&self) -> usize {
        unsafe { ffi::iio_context_get_attrs_count(self.inner.ctx) as usize }
    }

    /// Gets the name and value of the context-specific attributes.
    /// Note that this is different than the same function for other IIO
    /// types, in that this retrieves both the name and value of the
    /// attributes in a single call.
    pub fn get_attr(&self, idx: usize) -> Result<(String, String)> {
        let mut pname: *const c_char = ptr::null();
        let mut pval: *const c_char = ptr::null();

        sys_result(
            unsafe {
                ffi::iio_context_get_attr(self.inner.ctx, idx as c_uint, &mut pname, &mut pval)
            },
            (),
        )?;
        let name = cstring_opt(pname);
        let val = cstring_opt(pval);
        if name.is_none() || val.is_none() {
            return Err(Error::StringConversionError.into());
        }
        Ok((name.unwrap(), val.unwrap()))
    }

    /// Gets an iterator for the attributes in the context
    pub fn attributes(&self) -> AttrIterator<'_> {
        AttrIterator { ctx: self, idx: 0 }
    }

    /// Sets the timeout for I/O operations
    ///
    /// `timeout` The timeout. A value of zero specifies that no timeout
    ///     should be used.
    pub fn set_timeout(&self, timeout: Duration) -> Result<()> {
        let ms: u64 = 1000 * timeout.as_secs() + u64::from(timeout.subsec_millis());
        self.set_timeout_ms(ms)
    }

    /// Sets the timeout for I/O operations, in milliseconds
    ///
    /// `timeout` The timeout, in ms. A value of zero specifies that no
    ///     timeout should be used.
    pub fn set_timeout_ms(&self, ms: u64) -> Result<()> {
        let ret = unsafe { ffi::iio_context_set_timeout(self.inner.ctx, ms as c_uint) };
        sys_result(ret, ())
    }

    /// Get the number of devices in the context
    pub fn num_devices(&self) -> usize {
        unsafe { ffi::iio_context_get_devices_count(self.inner.ctx) as usize }
    }

    /// Gets a device by index
    pub fn get_device(&self, idx: usize) -> Result<Device> {
        let dev = unsafe { ffi::iio_context_get_device(self.inner.ctx, idx as c_uint) };
        if dev.is_null() {
            return Err(Error::InvalidIndex);
        }
        Ok(Device {
            dev,
            ctx: self.clone(),
        })
    }

    /// Try to find a device by name or ID
    /// `name` The name or ID of the device to find. For versions that
    /// support a label, it can also be used to look up a device.
    pub fn find_device(&self, name: &str) -> Option<Device> {
        let name = CString::new(name).unwrap();
        let dev = unsafe { ffi::iio_context_find_device(self.inner.ctx, name.as_ptr()) };
        if dev.is_null() {
            None
        }
        else {
            Some(Device {
                dev,
                ctx: self.clone(),
            })
        }
    }

    /// Gets an iterator for all the devices in the context.
    pub fn devices(&self) -> DeviceIterator<'_> {
        DeviceIterator { ctx: self, idx: 0 }
    }

    /// Destroy the context
    ///
    /// This consumes the context to destroy the instance.
    pub fn destroy(self) {}
}

impl PartialEq for Context {
    /// Two contexts are the same if they refer to the same underlying
    /// object in the library.
    fn eq(&self, other: &Self) -> bool {
        self.inner.ctx == other.inner.ctx
    }
}

impl From<InnerContext> for Context {
    /// Makes a new [`Context`] from the [`InnerContext`]
    fn from(inner: InnerContext) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

/// Iterator over the Devices in a Context
#[derive(Debug)]
pub struct DeviceIterator<'a> {
    /// Reference to the IIO context containing the Device
    ctx: &'a Context,
    /// The current Device index for the iterator
    idx: usize,
}

impl Iterator for DeviceIterator<'_> {
    type Item = Device;

    /// Gets the next Device from the iterator.
    fn next(&mut self) -> Option<Self::Item> {
        match self.ctx.get_device(self.idx) {
            Ok(dev) => {
                self.idx += 1;
                Some(dev)
            }
            Err(_) => None,
        }
    }
}

/// Iterator over the attributes in a Context
#[derive(Debug)]
pub struct AttrIterator<'a> {
    /// Reference to the IIO context containing the Device
    ctx: &'a Context,
    /// Index for the next Context attribute from the iterator
    idx: usize,
}

impl Iterator for AttrIterator<'_> {
    type Item = (String, String);

    /// Gets the next Device attribute from the iterator.
    fn next(&mut self) -> Option<Self::Item> {
        match self.ctx.get_attr(self.idx) {
            Ok(name_val) => {
                self.idx += 1;
                Some(name_val)
            }
            Err(_) => None,
        }
    }
}

// --------------------------------------------------------------------------
//                              Unit Tests
// --------------------------------------------------------------------------

// Note: These tests assume that the IIO Dummy kernel module is loaded
// locally with a device created. See the `load_dummy.sh` script.

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    // See that we get the default context.
    #[test]
    fn default_context() {
        let res = Context::new();
        assert!(res.is_ok());

        let res = Context::from_ptr(ptr::null_mut());
        assert!(res.is_err());
    }

    // Clone a context and make sure it's reported as same one.
    #[test]
    fn clone_context() {
        let ctx = Context::new().unwrap();
        let ctx2 = ctx.clone();
        assert_eq!(ctx, ctx2);
    }

    // Clone the inner context and send to another thread.
    #[test]
    fn multi_thread() {
        let ctx = Context::new().unwrap();
        let cti = ctx.try_clone_inner().unwrap();

        let thr = thread::spawn(move || {
            let _thr_ctx = Context::from_inner(cti);
        });
        thr.join().unwrap();
    }

    // See that device iterator gets the correct number of devices.
    #[test]
    fn dev_iterator_count() {
        let ctx = Context::new().unwrap();
        let ndev = ctx.num_devices();
        assert!(ndev != 0);
        assert!(ctx.devices().count() == ndev);
    }

    // See that the description gives back something.
    #[test]
    fn name() {
        let ctx = Context::new().unwrap();
        let name = ctx.name();
        println!("Context name: {}", name);
        assert!(name == "local" || name == "network");
    }

    // See that the description gives back something.
    #[test]
    fn description() {
        let ctx = Context::new().unwrap();
        let desc = ctx.description();
        println!("Context description: {}", desc);
        assert!(!desc.is_empty());
    }
}
