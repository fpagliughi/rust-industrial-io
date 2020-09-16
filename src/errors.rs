// libiio-sys/src/errors.rs
//
// Copyright (c) 2018-2020, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//!
//! Error definitions for the Industrial I/O Library.

use std::{io, result, ffi};
use nix;
use thiserror::Error;

//type SysError = nix::Error::Sys;

/// The Error type for the IIO library
#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    NulError(#[from] ffi::NulError),
    #[error("{0}")]
    Nix(#[from] nix::Error),
    #[error("Wrong data type")]
    WrongDataType,
    #[error("Bad return size")]
    BadReturnSize,
    #[error("Invalid index")]
    InvalidIndex,
    #[error("{0}")]
    General(String),
}

/// The default result type for the IIO library
pub type Result<T> = result::Result<T, Error>;

impl From<nix::errno::Errno> for Error {
    /// Converts a *nix errno into an Error
    fn from(err: nix::errno::Errno) -> Self {
        nix::Error::Sys(err).into()
    }
}

