// industrial-io/examples/riio_bufavg.rs
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
//!

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

// Active data processing object.
// Each one of these has a thread that can process a buffer's worth of
// incoming data at a time.
struct Averager {
    sender: Sender<TsDataBuffer>,
    thr: JoinHandle<()>,
}

impl Averager {
    // Creates a new averager with the specified offset and scale.
    pub fn new(offset: f64, scale: f64) -> Self {
        let (sender, receiver) = channel();
        let thr = spawn(move || Self::thread_func(receiver, offset, scale));
        Self { sender, thr }
    }

    // The internal thread function.
    // This just loops, receiving buffers of data, then averages them,
    // transforming to physical units like Volts, deg C, etc, and then
    // prints them to stdout.
    fn thread_func(receiver: Receiver<TsDataBuffer>, offset: f64, scale: f64) {
        loop {
            let (ts, data): TsDataBuffer = receiver.recv().unwrap();

            if data.is_empty() {
                break;
            }

            // Print the timestamp as the UTC time w/ millisec precision
            let sys_tm = SystemTime::UNIX_EPOCH + Duration::from_nanos(ts);
            let dt: DateTime<Utc> = sys_tm.into();
            print!("{}: ", dt.format("%T%.6f"));

            // Compute the average, then scale the result.
            let sum: f64 = data.iter().map(|&x| f64::from(x)).sum();
            let avg = sum / data.len() as f64;
            let val = (avg + offset) * scale / 1000.0;

            // Print out the scaled average, along with
            // the first few raw values from the buffer
            println!("<{:.2}> - {:?}", val, &data[0..4]);
        }
    }

    // Send data to the thread for processing
    pub fn send(&self, data: TsDataBuffer) -> Result<(), SendError<TsDataBuffer>> {
        self.sender.send(data)
    }

    // Tell the inner thread to quit, then block and wait for it.
    pub fn quit(self) {
        self.sender.send((0, vec![])).unwrap();
        self.thr.join().unwrap();
    }
}

/////////////////////////////////////////////////////////////////////////////

// If the IIO device doesn't have a timestamp channel, we can use this to
// get an equivalent, though less accurate, timestamp.
pub fn timestamp() -> u64 {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Clock error")
        .as_secs_f64();
    (1.0e9 * ts) as u64
}

/////////////////////////////////////////////////////////////////////////////

fn run() -> Result<()> {
    let args = Command::new("riio_bufavg")
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
            arg!(-t --trigger <trigger> "Specifies the name of the trigger").action(ArgAction::Set),
            arg!(-n --num_sample <num_sample> "Specifies the number of samples per buffer")
                .action(ArgAction::Set)
                .value_parser(value_parser!(usize)),
            arg!(-f --frequency <frequency> "Specifies the sampling frequency")
                .action(ArgAction::Set)
                .value_parser(value_parser!(i64)),
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

    println!("Using device: {}", dev_name);

    // ----- Find the timestamp channel (if any) and a data channel -----

    let mut ts_chan = dev.find_channel("timestamp", iio::Direction::Input);

    if ts_chan.is_some() {
        println!("Found timestamp channel.");
    }
    else {
        println!("No timestamp channel. Estimating timestamps.");
    }

    let sample_chan = dev
        .find_channel(chan_name, iio::Direction::Input)
        .with_context(|| format!("No '{}' channel on this device", chan_name))?;

    println!("Using channel: {}", chan_name);

    if sample_chan.type_of() != Some(TypeId::of::<RawSampleType>()) {
        bail!(
            "The channel type ({:?}) is different than expected.",
            sample_chan.type_of()
        );
    }

    if let Some(ref mut chan) = ts_chan {
        chan.enable();
    }

    sample_chan.enable();

    // ----- Check for a scale and offset -----

    let offset: f64 = sample_chan.attr_read_float("offset").unwrap_or(0.0);
    let scale: f64 = sample_chan.attr_read_float("scale").unwrap_or(1.0);

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
                .copied()
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

        let data: Vec<RawSampleType> = buf.channel_iter::<RawSampleType>(&sample_chan)
                                           .map(|x| sample_chan.convert(x))
                                           .collect();
        */

        let data: Vec<RawSampleType> = match sample_chan.read(&buf) {
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

    Ok(())
}

// --------------------------------------------------------------------------

fn main() {
    if let Err(err) = run() {
        eprintln!("{:#}", err);
        process::exit(1);
    }
}
