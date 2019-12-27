// industrial-io/examples/riio_readraw.rs
//
// Periodically reads the samples from all channels on a device that have
// the "raw" attribute. This periodically polls the channels using a Rust
// timer via the schedule_recv crate.
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

#[macro_use] extern crate clap;
extern crate industrial_io as iio;
extern crate schedule_recv;

use std::process;
use std::time::Duration;
use schedule_recv::periodic;
use clap::{Arg, App};

fn main() -> iio::Result<()> {
    let matches = App::new("riio_readraw")
                    .version(crate_version!())
                    .about("Rust IIO raw reads example.")
                    .arg(Arg::with_name("device")
                         .short("d")
                         .long("device")
                         .help("Specifies the name of the IIO device to read")
                         .takes_value(true))
                    .arg(Arg::with_name("network")
                         .short("n")
                         .long("network")
                         .help("Use the network backend with the provided hostname")
                         .takes_value(true))
                    .arg(Arg::with_name("uri")
                         .short("u")
                         .long("uri")
                         .help("Use the context with the provided URI")
                         .takes_value(true))
                    .get_matches();

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

    let dev = if let Some(dev_name) = matches.value_of("device") {
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
            if let Ok(val) = chan.attr_read_int("raw") {
                print!(" {:>8} ", val);
            }
        }
        println!();
    }
}

