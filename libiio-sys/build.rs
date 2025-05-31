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

use std::{env, path::PathBuf};

use pkg_config::{self, Library};

fn main() {
    let lib = pkg_config::Config::new()
        .range_version("0.19"..="0.25")
        .probe("libiio")
        .expect("Failed to find an acceptable version of libiio");
    let Library { include_paths, .. } = lib;
    let mut include_args = vec![];

    for inc_path in include_paths.iter() {
        include_args.push(format!("-I{}", inc_path.to_str().unwrap()));
    }

    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_args(include_args)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    let bindings = bindings.generate().unwrap();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("iio_bindings.rs"))
        .expect("Couldn't write bindings!");
}
