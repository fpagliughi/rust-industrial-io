// industrial-io/examples/riio_detect.rs
//
// This example is part of the Rust industrial-io crate.
//
// Copyright (c) 2018-2025, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//! Simple Rust IIO example to list the devices found in the specified context.
//!
//! Note that, if no context is requested at the command line, this will create
//! a network context if the IIOD_REMOTE environment variable is set, otherwise
//! it will create a local context. See Context::new().
//!

use clap::{arg, ArgAction, Command};
use industrial_io as iio;
use std::process;

fn main() {
    let args = Command::new("riio_detect")
        .version(clap::crate_version!())
        .about("Rust IIO free scan buffered reads.")
        .args(&[
            arg!(-h --host <host> "Use the network backend with the specified host")
                .action(ArgAction::Set),
            arg!(-u --uri <uri> "Use the context with the provided URI")
                .action(ArgAction::Set)
                .conflicts_with("host"),
            arg!(-'v' --version "Print version information").action(ArgAction::Version),
            arg!(-'?' --help "Print help information")
                .global(true)
                .action(ArgAction::Help),
        ])
        .get_matches();

    let ctx = if let Some(host) = args.get_one::<String>("host") {
        println!("Using host: {}", host);
        iio::Context::with_backend(iio::Backend::Network(host))
    }
    else if let Some(uri) = args.get_one::<String>("uri") {
        println!("Using URI: {}", uri);
        iio::Context::from_uri(uri)
    }
    else {
        iio::Context::new()
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
