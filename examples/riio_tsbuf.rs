// industrial-io/examples/riio_tsbuf.rs
//
// Simple Rust IIO example for time-stamped, buffered, reading
// using a trigger.
//
// This example requires a device with a timestamp channel.
//
// Copyright (c) 2018-2021, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

#[macro_use]
extern crate clap;

use anyhow::{bail, Context, Result};
use chrono::{offset::Utc, DateTime};
use clap::{App, Arg};
use industrial_io as iio;
use std::{
    cmp, process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};

const DFLT_DEV_NAME: &str = "ads1015";
const DFLT_CHAN_NAME: &str = "voltage0";

const SAMPLING_FREQ_ATTR: &str = "sampling_frequency";

const DFLT_FREQ: i64 = 100;
const DFLT_NUM_SAMPLE: usize = 100;

// --------------------------------------------------------------------------

fn run() -> Result<()> {
    let args = App::new("riio_tsbuf")
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

    let dev_name = args.value_of("device").unwrap_or(DFLT_DEV_NAME);
    let chan_name = args.value_of("channel").unwrap_or(DFLT_CHAN_NAME);

    let ctx = if let Some(hostname) = args.value_of("host") {
        iio::Context::with_backend(iio::Backend::Network(hostname))
    }
    else if let Some(uri) = args.value_of("uri") {
        iio::Context::from_uri(uri)
    }
    else {
        iio::Context::new()
    }
    .context("Couldn't open IIO context.")?;

    let dev = ctx
        .find_device(dev_name)
        .context(format!("No IIO device named '{}'", dev_name))?;

    // ----- Find the timestamp channel and a voltage channel -----

    let ts_chan = dev
        .find_channel("timestamp", false)
        .context("No timestamp channel on this device")?;

    let sample_chan = dev
        .find_channel(chan_name, false)
        .context(format!("No '{}' channel on this device", chan_name))?;

    ts_chan.enable();
    sample_chan.enable();

    // ----- Set sample frequency and trigger -----

    let freq = args
        .value_of("frequency")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(DFLT_FREQ);

    // If the user asked for a trigger device, see if we can use it
    if let Some(trig_name) = args.value_of("trigger") {
        let trig = ctx
            .find_device(trig_name)
            .context(format!("Couldn't find requested trigger: {}", trig_name))?;

        // Set the sampling rate on the trigger device
        trig.attr_write(SAMPLING_FREQ_ATTR, freq)
            .with_context(|| format!("Can't set sampling rate to {}Hz on {}", freq, trig_name))?;

        dev.set_trigger(&trig)
            .context("Error setting the trigger on the device")?;
    }
    else if dev.has_attr(SAMPLING_FREQ_ATTR) {
        // Try to set the sampling rate on the device itself, if supported
        dev.attr_write(SAMPLING_FREQ_ATTR, freq)
            .with_context(|| {
                format!(
                    "Can't set sampling rate to {}Hz on {}",
                    freq,
                    dev.name().unwrap()
                )
            })?;
    }
    else {
        bail!("No suitable trigger device found");
    }

    // ----- Create a buffer -----

    let n_sample = args
        .value_of("num_sample")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(DFLT_NUM_SAMPLE);

    let buf = dev
        .create_buffer(n_sample, false)
        .context("Unable to create buffer")?;

    // Make sure the timeout is more than enough to gather each buffer
    // Give 50% extra time, or at least 5sec.
    let ms = cmp::max(5000, 1500 * (n_sample as u64) / (freq as u64));
    if let Err(err) = ctx.set_timeout_ms(ms) {
        eprintln!("Error setting timeout of {}ms: {}", ms, err);
    }

    // ---- Handle ^C for a graceful shutdown -----

    let quit = Arc::new(AtomicBool::new(false));
    let q = quit.clone();

    ctrlc::set_handler(move || {
        q.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // ----- Capture data into the buffer -----

    println!("Staring buffer capture...");

    while !quit.load(Ordering::SeqCst) {
        buf.refill().context("Error filling the buffer")?;

        // Extract and print the data

        let ts_data = buf.channel_iter::<u64>(&ts_chan);

        // The timestamp is represented as a 64-bit integer number of
        // nanoseconds since the Unix Epoch. We convert to a Rust SystemTime,
        // then a chrono DataTime for pretty printing.
        buf.channel_iter::<u16>(&sample_chan)
            .zip(ts_data.map(|ts| {
                DateTime::<Utc>::from(SystemTime::UNIX_EPOCH + Duration::from_nanos(ts))
                    .format("%T%.6f")
            }))
            .for_each(|(data, time)| println!("{}: {}", time, data));
    }

    Ok(())
}

// --------------------------------------------------------------------------

fn main() {
    if let Err(err) = run() {
        eprintln!("{:#}", err);
        process::exit(1);
    }
}
