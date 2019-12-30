// libiio-sys/src/macros.rs
//
// Copyright (c) 2019, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//! Industrial I/O Buffers
//!

#![macro_use]

macro_rules! cstring_or_bail {
    ($name:expr) => {
        match CString::new($name) {
            Ok(s) => s,
            Err(_) => return None,
        }
    }
}

macro_rules! cstring_or_bail_false {
    ($name:expr) => {
        match CString::new($name) {
            Ok(s) => s,
            Err(_) => return false,
        }
    }
}
