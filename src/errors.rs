// libiio-sys/src/errors.rs
//
// Copyright (c) 2018-2025, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//!
//! Error definitions for the Industrial I/O Library.

use std::{ffi, io};
use thiserror::Error;

/// The Error type for the IIO library
#[derive(Error, Debug)]
pub enum Error {
    /// A low-level I/O error
    #[error("{0}")]
    Io(#[from] io::Error),
    /// An unexpected NUL value returned from the C library.
    #[error("{0}")]
    NulError(#[from] ffi::NulError),
    /// A low-level Unix-style error
    #[error("{0}")]
    Nix(#[from] nix::Error),
    /// An error converting a value to/from a string representation.
    #[error("String conversion error")]
    StringConversionError,
    /// The wrong data type used in an operation
    #[error("Wrong data type")]
    WrongDataType,
    /// The size of a data or return value was different than expected.
    #[error("Bad return size")]
    BadReturnSize,
    /// A device or channel index did not find a requested object
    #[error("Invalid index")]
    InvalidIndex,
    /// A generic error with a string explanation
    #[error("{0}")]
    General(String),
}

/// The default result type for the IIO library
pub type Result<T> = std::result::Result<T, Error>;
