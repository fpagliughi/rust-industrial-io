// industrial-io/examples/riio_chan_info.rs
//
// This example is part of the Rust industrial-io crate.
//
// Copyright (c) 2019-2025, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//! Simple Rust IIO example for buffered reading and post-processing.
//!
//! This does buffered reading with a trigger, then sends the data
//! to a second thread to convert and process.
//!
//! For the sake of simplicity, we assume a raw sample type of a signed,
//! 16-bit integer. A real, dedicated application might do something similar,
//! but a general-purpose solution would probe the channel type and/or use
//! generics to read and convert the raw data.
//!
//! For quick tests, just set `RawSampleType` to the type matching the channel
//! to be tested.
//

// TODO: Get rid of this and clean up
#![allow(unused_imports, dead_code)]

use anyhow::{bail, Context, Result};
use chrono::{offset::Utc, DateTime};
use clap::{arg, value_parser, ArgAction, Command};
use industrial_io as iio;
use std::{
    any::TypeId,
    cmp, process,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, SendError, Sender},
        Arc,
    },
    thread::{spawn, JoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

// The type to use for raw samples.
type RawSampleType = i16;

// Time-stamped data buffer
type TsDataBuffer = (u64, Vec<RawSampleType>);

// The defaults device and channel if none specified
const DFLT_DEV_NAME: &str = "ads1015";
const DFLT_CHAN_NAME: &str = "voltage0";

const DFLT_FREQ: i64 = 100;
const DFLT_NUM_SAMPLE: usize = 100;

const SAMPLING_FREQ_ATTR: &str = "sampling_frequency";

/////////////////////////////////////////////////////////////////////////////

fn run() -> Result<()> {
    let args = Command::new("riio_chan_info")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("Rust IIO timestamped buffered read & average example.")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .args(&[
            arg!(-h --host <host> "Use the network backend with the specified host")
                .action(ArgAction::Set),
            arg!(-u --uri <uri> "Use the context with the provided URI").action(ArgAction::Set),
            arg!(-d --device <device> "Specifies the name of the IIO device to read")
                .default_value(DFLT_DEV_NAME),
            arg!(-c --channel <channel> "Specifies the name of the channel to read")
                .default_value(DFLT_CHAN_NAME),
            arg!(-'v' --version "Print version information").action(ArgAction::Version),
            arg!(-'?' --help "Print help information")
                .global(true)
                .action(ArgAction::Help),
        ])
        .get_matches();

    let dev_name = args.get_one::<String>("device").unwrap();
    let chan_name = args.get_one::<String>("channel").unwrap();

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
    .context("Couldn't open IIO context.")?;

    let dev = ctx
        .find_device(dev_name)
        .with_context(|| format!("No IIO device named '{}'", dev_name))?;

    println!("Device: {}", dev_name);

    // ----- Find the channel -----

    let none = String::from("<none>");

    let chan = dev
        .find_channel(chan_name, iio::Direction::Input)
        .with_context(|| format!("No '{}' channel on this device", chan_name))?;

    println!("\nChannel: {}", chan_name);

    println!("  Name: {}", chan.name().unwrap_or(none.clone()));
    println!("  Id: {}", chan.id().unwrap_or(none.clone()));
    println!("  Dir: {:?}", chan.direction());
    println!("  Scan Element: {}", chan.is_scan_element());
    if let Ok(idx) = chan.index() {
        println!("  Index: {}", idx);
    }
    println!("  Attributes: [{}]", chan.num_attrs());
    for attr in chan.attrs() {
        print!("    {}", attr);
        if let Ok(sval) = chan.attr_read_str(&attr) {
            print!(": {}", sval);
        }
        println!();
    }

    /*
    if chan.type_of() != Some(TypeId::of::<RawSampleType>()) {
        bail!(
            "The channel type ({:?}) is different than expected.",
            chan.type_of()
        );
    }
    */

    /*
        // ----- Check for a scale and offset -----

        let offset: f64 = chan.attr_read_float("offset").unwrap_or(0.0);
        let scale: f64 = chan.attr_read_float("scale").unwrap_or(1.0);

        println!("  Offset: {:.3}, Scale: {:.3}", offset, scale);

        // ----- Set sample frequency and trigger -----

        let freq = *args.get_one("frequency").unwrap_or(&DFLT_FREQ);

        // If the user asked for a trigger device, see if we can use it
        if let Some(trig_name) = args.get_one::<String>("trigger") {
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
            dev.attr_write(SAMPLING_FREQ_ATTR, freq).with_context(|| {
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

        let n_sample = *args.get_one("num_sample").unwrap_or(&DFLT_NUM_SAMPLE);

        let mut buf = dev
            .create_buffer(n_sample, false)
            .context("Unable to create buffer")?;

        // Make sure the timeout is more than enough to gather each buffer
        // Give 50% extra time, or at least 5sec.
        let ms = cmp::max(5000, 1500 * (n_sample as u64) / (freq as u64));
        if let Err(err) = ctx.set_timeout_ms(ms) {
            eprintln!("Error setting timeout of {}ms: {}", ms, err);
        }

        // ----- Create the averager -----

        let avg = Averager::new(offset, scale);

        // ---- Handle ^C since we want a graceful shutdown -----

        let quit = Arc::new(AtomicBool::new(false));
        let q = quit.clone();

        ctrlc::set_handler(move || {
            q.store(true, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

        // ----- Capture data into the buffer -----

        println!("Started capturing data...");

        while !quit.load(Ordering::SeqCst) {
            buf.refill().context("Error filling the buffer")?;

            // Get the timestamp. Use the time of the _last_ sample.

            let ts: u64 = if let Some(ref chan) = ts_chan {
                buf.channel_iter::<u64>(chan)
                    .nth(n_sample - 1)
                    .map(|&x| x)
                    .unwrap_or_default()
            }
            else {
                timestamp()
            };

            // Extract and convert the raw data from the buffer.
            // This puts the raw samples into host format (fixes "endianness" and
            // shifts into place), but it's still raw data. The other thread
            // will apply the offset and scaling.
            // We do this here because the channel is not thread-safe.

            /*
            Note: We could do the following to convert each sample, one at a time,
                but it's more efficient to convert the whole buffer using read()

            let data: Vec<RawSampleType> = buf.channel_iter::<RawSampleType>(&chan)
                                               .map(|x| chan.convert(x))
                                               .collect();
            */

            let data: Vec<RawSampleType> = match chan.read(&buf) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("Error reading data: {}", err);
                    break;
                }
            };

            avg.send((ts, data)).unwrap();
        }

        // ----- Shut down -----

        println!("\nExiting...");
        avg.quit();
        println!("Done");
    */
    Ok(())
}

// --------------------------------------------------------------------------

fn main() {
    if let Err(err) = run() {
        eprintln!("{:#}", err);
        process::exit(1);
    }
}
