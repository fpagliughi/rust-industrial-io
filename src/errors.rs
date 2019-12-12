// libiio-sys/src/errors.rs
//
// Copyright (c) 2018, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//!
//! Error definitions for the Industrial I/O Library.

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        NulError(::std::ffi::NulError);
        Nix(::nix::Error);
    }
}

