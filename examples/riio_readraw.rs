// industrial-io/examples/riio_readraw.rs
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

//! Example to read raw analog data.
//!
//! Periodically reads the samples from all channels on a device that have
//! the "raw" attribute. This periodically polls the channels using a Rust
//! timer via the schedule_recv crate.
//!
//! Note that, if no context is requested at the command line, this will create
//! a network context if the IIOD_REMOTE environment variable is set, otherwise
//! it will create a local context. See Context::new().
//

use clap::{arg, ArgAction, Command};
use industrial_io as iio;
use schedule_recv::periodic;
use std::{process, time::Duration};

fn main() -> iio::Result<()> {
    let args = Command::new("riio_readraw")
        .version(clap::crate_version!())
        .about("Rust IIO raw reads example.")
        .args(&[
            arg!(-h --host <host> "Use the network backend with the specified host")
                .action(ArgAction::Set),
            arg!(-u --uri <uri> "Use the context with the provided URI")
                .action(ArgAction::Set)
                .conflicts_with("host"),
            arg!(-d --device <device> "Specifies the name of the IIO device to read"),
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

    let dev = if let Some(dev_name) = args.get_one::<String>("device") {
        ctx.find_device(dev_name).unwrap_or_else(|| {
            eprintln!("Couldn't find device: {}", dev_name);
            process::exit(1);
        })
    }
    else {
        ctx.get_device(0).unwrap_or_else(|err| {
            eprintln!("Error opening first device: {}", err);
            process::exit(1);
        })
    };

    let unknown = "unknown".to_string();
    let tick = periodic(Duration::from_millis(1000));

    println!("Device: {}", dev.name().unwrap_or_else(|| unknown.clone()));

    for chan in dev.channels() {
        if chan.has_attr("raw") {
            print!(" {:>9}", chan.id().unwrap_or_else(|| unknown.clone()));
        }
    }
    println!();

    loop {
        tick.recv().unwrap();
        for chan in dev.channels() {
            match chan.attr_read::<i64>("raw") {
                Ok(val) => print!(" {:>8} ", val),
                _ => print!("######## "),
            }
        }
        println!();
    }
}
