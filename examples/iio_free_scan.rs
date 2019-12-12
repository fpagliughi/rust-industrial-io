// industrial-io/examples/iio_free_scan.rs
//
// Simple Rust IIO example.
// This does buffered reading without using a trigger (free scan).
//
// Copyright (c) 2018-2019, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

extern crate industrial_io as iio;

#[macro_use]
extern crate clap;

use std::process;
use clap::{Arg, App};

const DFLT_DEV_NAME: &'static str = "44e0d000.tscadc:adc";

fn main() {
    let matches = App::new("iio_free_scan")
                    .version(crate_version!())
                    .about("IIO free scan buffered reads.")
                    .arg(Arg::with_name("device")
                         .short("d")
                         .long("device")
                         .value_name("DEVICE")
                         .help("Specifies the name of the IIO device to read")
                         .takes_value(true))
                     .get_matches();

    let dev_name = matches.value_of("device").unwrap_or(DFLT_DEV_NAME);

    println!("Device: {}", dev_name);

    let ctx = iio::Context::new().unwrap_or_else(|_err| {
        println!("Couldn't open default IIO context");
        process::exit(1);
    });

    let dev = ctx.find_device(dev_name).unwrap_or_else(|| {
        println!("No IIO device named '{}'", dev_name);
        process::exit(2);
    });

    for mut chan in dev.channels() {
        chan.enable();
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

