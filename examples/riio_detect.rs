// industrial-io/examples/riio_detect.rs
//
// Simple Rust IIO example to list the devices found in the specified context.
//
// Note that, if no context is requested at the command line, this will create
// a network context if the IIOD_REMOTE environment variable is set, otherwise
// it will create a local context. See Context::new().
//
// Copyright (c) 2018-2019, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

#[macro_use]
extern crate clap;

use clap::{App, Arg};
use industrial_io as iio;
use std::process;

fn main() {
    let matches = App::new("riio_free_scan")
        .version(crate_version!())
        .about("Rust IIO free scan buffered reads.")
        .arg(
            Arg::with_name("network")
                .short("n")
                .long("network")
                .help("Use the network backend with the provided hostname")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("uri")
                .short("u")
                .long("uri")
                .help("Use the context with the provided URI")
                .takes_value(true),
        )
        .get_matches();

    let ctx = if let Some(hostname) = matches.value_of("network") {
        iio::Context::new(iio::Backend::Ip(hostname))
    }
    else if let Some(uri) = matches.value_of("uri") {
        iio::Context::new(iio::Backend::FromUri(uri))
    }
    else {
        iio::Context::default()
    }
    .unwrap_or_else(|_err| {
        println!("Couldn't open IIO context.");
        process::exit(1);
    });

    let mut trigs = Vec::new();

    if ctx.num_devices() == 0 {
        println!("No devices in the default IIO context");
    }
    else {
        println!("IIO Devices:");
        for dev in ctx.devices() {
            if dev.is_trigger() {
                if let Some(id) = dev.id() {
                    trigs.push(id);
                }
            }
            else {
                print!("  {} ", dev.id().unwrap_or_default());
                print!("[{}]", dev.name().unwrap_or_default());
                println!(": {} channel(s)", dev.num_channels());
            }
        }

        if !trigs.is_empty() {
            println!("\nTriggers Devices:");
            for s in trigs {
                println!("  {}", s);
            }
        }
    }
}
