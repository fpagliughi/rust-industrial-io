// industrial-io/examples/riio_free_scan.rs
//
// Simple Rust IIO example for buffered, free-scan reading.
// This does buffered reading without using a trigger.
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
extern crate industrial_io as iio;

use clap::{App, Arg};
use std::any::TypeId;
use std::process;

const DFLT_DEV_NAME: &str = "44e0d000.tscadc:adc";

fn main() {
    let matches = App::new("riio_free_scan")
        .version(crate_version!())
        .about("Rust IIO free scan buffered reads.")
        .arg(
            Arg::with_name("device")
                .short("d")
                .long("device")
                .help("Specifies the name of the IIO device to read")
                .takes_value(true),
        )
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

    let dev_name = matches.value_of("device").unwrap_or(DFLT_DEV_NAME);

    let ctx = if let Some(hostname) = matches.value_of("network") {
        iio::Context::create_network(hostname)
    }
    else if let Some(uri) = matches.value_of("uri") {
        iio::Context::create_from_uri(uri)
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
    for mut chan in dev.channels() {
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
        let data: Vec<u16> = buf.channel_iter::<u16>(&chan).collect();
        println!("{}: {:?}", chan.id().unwrap_or_default(), data);
    }
}
