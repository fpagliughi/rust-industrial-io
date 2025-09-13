// industrial-io/examples/riio_free_scan.rs
//
// This example is part of the Rust industrial-io crate.
//
// Copyright (c) 2018-2025, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.

//! Simple Rust IIO example for buffered, free-scan reading.
//!
//! This does buffered reading without using a trigger.
//!

use clap::{arg, ArgAction, Command};
use industrial_io as iio;
use std::{any::TypeId, process};

const DFLT_DEV_NAME: &str = "44e0d000.tscadc:adc";

fn main() {
    let args = Command::new("riio_free_scan")
        .version(clap::crate_version!())
        .about("Rust IIO free scan buffered reads.")
        .args(&[
            arg!(-h --host <host> "Use the network backend with the specified host")
                .action(ArgAction::Set),
            arg!(-u --uri <uri> "Use the context with the provided URI")
                .action(ArgAction::Set)
                .conflicts_with("host"),
            arg!(-d --device <device> "Specifies the name of the IIO device to read")
                .default_value(DFLT_DEV_NAME),
            arg!(-'v' --version "Print version information").action(ArgAction::Version),
            arg!(-'?' --help "Print help information")
                .global(true)
                .action(ArgAction::Help),
        ])
        .get_matches();

    let dev_name = args.get_one::<String>("device").unwrap();

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

    let dev = ctx.find_device(dev_name).unwrap_or_else(|| {
        println!("No IIO device named '{}'", dev_name);
        process::exit(2);
    });

    // Note that we just look at unsigned 16-bit samples.
    // This is arbitrary for the purpose of example.

    let mut nchan = 0;
    for chan in dev.channels() {
        if chan.type_of() == Some(TypeId::of::<u16>()) {
            nchan += 1;
            chan.enable();
        }
    }

    if nchan == 0 {
        eprintln!("Couldn't find any unsigned 16-bit channels to capture.");
        process::exit(2);
    }

    let mut buf = dev.create_buffer(8, false).unwrap_or_else(|err| {
        eprintln!("Unable to create buffer: {}", err);
        process::exit(3);
    });

    println!("Capturing a buffer...");
    if let Err(err) = buf.refill() {
        eprintln!("Error filling the buffer: {}", err);
        process::exit(4);
    }

    for chan in dev.channels() {
        let data: Vec<u16> = buf.channel_iter::<u16>(&chan).copied().collect();
        println!("{}: {:?}", chan.id().unwrap_or_default(), data);
    }
}
