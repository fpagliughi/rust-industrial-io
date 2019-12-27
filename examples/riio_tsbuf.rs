// industrial-io/examples/riio_tsbuf.rs
//
// Simple Rust IIO example for time-stamped, buffered, reading.
// This does buffered reading with a trigger.
//
// This example requires a A/D with a timestamp channel.
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
extern crate chrono;

use std::process;
use std::time::{SystemTime, Duration};
use clap::{Arg, App};
use chrono::offset::Utc;
use chrono::DateTime;

const DFLT_DEV_NAME: &'static str = "ads1015";
const DFLT_CHAN_NAME: &'static str = "voltage0";

fn main() {
    let matches = App::new("riio_tsbuf")
                    .version(crate_version!())
                    .about("Rust IIO timestamped buffered read example.")
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
                    .arg(Arg::with_name("device")
                        .short("d")
                        .long("device")
                        .help("Specifies the name of the IIO device to read")
                        .takes_value(true))
                    .arg(Arg::with_name("channel")
                        .short("c")
                        .long("channel")
                        .help("Specifies the name of the channel to read")
                        .takes_value(true))
                    .get_matches();

    let dev_name = matches.value_of("device").unwrap_or(DFLT_DEV_NAME);
    let chan_name = matches.value_of("chan").unwrap_or(DFLT_CHAN_NAME);

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

    let mut dev = ctx.find_device(dev_name).unwrap_or_else(|| {
        println!("No IIO device named '{}'", dev_name);
        process::exit(2);
    });

    // ----- Find the timestamp channel and a voltage channel -----

    let mut ts_chan = dev.find_channel("timestamp", false).unwrap_or_else(|| {
        println!("No timestamp channel on this device");
        process::exit(1);
    });

    let mut sample_chan = dev.find_channel(chan_name, false).unwrap_or_else(|| {
        println!("No '{}' channel on this device", chan_name);
        process::exit(1);
    });

    ts_chan.enable();
    sample_chan.enable();

    println!("Sample size: {}", dev.sample_size().unwrap());

    // ----- Set a trigger -----

    // TODO: Make this a cmd-line option
    const TRIGGER: &'static str = "trigger0";
    const RATE_HZ: i64 = 100;

    let trig = ctx.find_device(TRIGGER).unwrap_or_else(|| {
        eprintln!("Couldn't find requested trigger: {}", TRIGGER);
        process::exit(1);
    });

    // Set the sampling rate
    if let Err(err) = trig.attr_write_int("sampling_frequency", RATE_HZ) {
        println!("Can't set sampling rate: {}", err);
    }

    dev.set_trigger(&trig).unwrap_or_else(|err| {
        println!("Error setting the trigger in the device: {}", err);
        process::exit(2);
    });

    // ----- Create a buffer -----

    let mut buf = dev.create_buffer(200, false).unwrap_or_else(|err| {
        eprintln!("Unable to create buffer: {}", err);
        process::exit(3);
    });

    // ----- Capture data into the buffer -----

    println!("Capturing a buffer...");
    if let Err(err) = buf.refill() {
        eprintln!("Error filling the buffer: {}", err);
        process::exit(4);
    }

    // Extract and print the data

    let mut ts_data = buf.channel_iter::<u64>(&ts_chan);
    let mut sample_data = buf.channel_iter::<u16>(&sample_chan);

    loop {
        // Get the next timestamp. It's represented as the 64-bit integer
        // number of nanoseconds since the Unix Epoch. We convert to a
        // Rust SystemTime, then a chrono DataTime for pretty printing.
        if let Some(ts) = ts_data.next() {
            let sys_tm = SystemTime::UNIX_EPOCH + Duration::from_nanos(ts);
            let dt: DateTime<Utc> = sys_tm.into();
            print!("[{}]: ", dt.format("%T%.6f"));
        }
        else {
            break;
        }
        if let Some(val) = sample_data.next() {
            print!("{}", val);
        }
        println!();
    }
}

