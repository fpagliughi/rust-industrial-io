// libiio-sys/build.rs
//
// The builder for the Linux Industrial I/O wrapper crate.
//
// Copyright (c) 2018-2022, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

use std::env;

fn config_macos() {
    println!("cargo:rustc-link-lib=framework=iio");

    if cfg!(target_arch = "x86_64") {
        println!(r"cargo:rustc-link-search=framework=/usr/local/Frameworks/");
    }
    else {
        println!(r"cargo:rustc-link-search=framework=/opt/homebrew/Frameworks/");
    }
}

fn main() {
    // TODO: We should eventually find or regenerate the
    //      bindings file for the specific target.
    let tgt = env::var("TARGET").unwrap();
    println!("debug: Building for target: '{}'", tgt);

    #[cfg(feature = "libiio_v0_25")]
    println!("debug: Using bindings for libiio v0.25");

    #[cfg(feature = "libiio_v0_24")]
    println!("debug: Using bindings for libiio v0.24");

    #[cfg(feature = "libiio_v0_23")]
    println!("debug: Using bindings for libiio v0.23");

    #[cfg(feature = "libiio_v0_21")]
    println!("debug: Using bindings for libiio v0.21");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "linux" {
        println!("cargo:rustc-link-lib=iio");
    }
    else if target_os == "macos" {
        config_macos();
    }
}
