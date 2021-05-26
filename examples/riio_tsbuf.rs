// industrial-io/examples/riio_tsbuf.rs
//
// Simple Rust IIO example for time-stamped, buffered, reading
// using a trigger.
//
// This example requires a device with a timestamp channel.
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
extern crate chrono;
extern crate industrial_io as iio;

use chrono::offset::Utc;
use chrono::DateTime;
use clap::{App, Arg};
use std::time::{Duration, SystemTime};
use std::{cmp, process};

const DFLT_DEV_NAME: &str = "ads1015";
const DFLT_CHAN_NAME: &str = "voltage0";
const DFLT_TRIG_NAME: &str = "trigger0";

const DFLT_FREQ: i64 = 100;
const DFLT_NUM_SAMPLE: usize = 100;

// --------------------------------------------------------------------------

fn main() {
    let matches = App::new("riio_tsbuf")
        .version(crate_version!())
        .about("Rust IIO timestamped buffered read example.")
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
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
        .arg(
            Arg::with_name("device")
                .short("d")
                .long("device")
                .help("Specifies the name of the IIO device to read")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("channel")
                .short("c")
                .long("channel")
                .help("Specifies the name of the channel to read")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("trigger")
                .short("t")
                .long("trigger")
                .help("Specifies the name of the trigger")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("num_sample")
                .short("n")
                .long("num_sample")
                .help("Specifies the number of samples per buffer")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("frequency")
                .short("f")
                .long("frequency")
                .help("Specifies the sampling frequency")
                .takes_value(true),
        )
        .get_matches();

    let dev_name = matches.value_of("device").unwrap_or(DFLT_DEV_NAME);
    let chan_name = matches.value_of("channel").unwrap_or(DFLT_CHAN_NAME);
    let trig_name = matches.value_of("trigger").unwrap_or(DFLT_TRIG_NAME);

    let mut ctx = if let Some(hostname) = matches.value_of("host") {
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

    // ----- Set a trigger -----

    let trig = ctx.find_device(trig_name).unwrap_or_else(|| {
        eprintln!("Couldn't find requested trigger: {}", trig_name);
        process::exit(1);
    });

    let freq = matches
        .value_of("frequency")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(DFLT_FREQ);

    // Set the sampling rate
    if let Err(err) = trig.attr_write_int("sampling_frequency", freq) {
        println!("Can't set sampling rate to {}Hz: {}", freq, err);
    }

    dev.set_trigger(&trig).unwrap_or_else(|err| {
        println!("Error setting the trigger in the device: {}", err);
        process::exit(2);
    });

    // ----- Create a buffer -----

    let n_sample = matches
        .value_of("num_sample")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(DFLT_NUM_SAMPLE);

    let mut buf = dev.create_buffer(n_sample, false).unwrap_or_else(|err| {
        eprintln!("Unable to create buffer: {}", err);
        process::exit(3);
    });

    // Make sure the timeout is more than enough to gather each buffer
    // Give 50% extra time, or at least 5sec.
    let ms = cmp::max(5000, 1500 * (n_sample as u64) / (freq as u64));
    if let Err(err) = ctx.set_timeout_ms(ms) {
        eprintln!("Error setting timeout of {}ms: {}", ms, err);
    }

    // ----- Capture data into the buffer -----

    println!("Capturing a buffer...");
    if let Err(err) = buf.refill() {
        eprintln!("Error filling the buffer: {}", err);
        process::exit(4);
    }

    // Extract and print the data

    let ts_data = buf.channel_iter::<u64>(&ts_chan);
    let mut sample_data = buf.channel_iter::<u16>(&sample_chan);

    // The timestamp is represented as a 64-bit integer number of
    // nanoseconds since the Unix Epoch. We convert to a Rust SystemTime,
    // then a chrono DataTime for pretty printing.
    sample_data.zip(
            ts_data.map(|ts| DateTime::<Utc>::from(
                    SystemTime::UNIX_EPOCH + Duration::from_nanos(ts))
                    .format("%T%.6f")
        ))
        .for_each(|(data, time)| println!("{}: {}", time, data));
}
