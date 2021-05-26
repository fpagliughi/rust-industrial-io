// industrial-io/examples/riio_bufavg.rs
//
// Simple Rust IIO example for buffered reading and post-processing.
// This does buffered reading with a trigger, then sends the data
// to a second thread to convert and process.
//
// For the sake of simplicity, we assume a raw sample type of a signed,
// 16-bit integer. A real, dedicated application might do something similar,
// but a general-purpose solution would probe the channel type and/or use
// generics to read and convert the raw data.
//
// For quick tests, just set `RawSampleType` to the type matchine the channel
// to be tested.
//
// Copyright (c) 2019, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

#[macro_use]
extern crate clap;

use chrono::offset::Utc;
use chrono::DateTime;
use clap::{App, Arg};
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
const DFLT_TRIG_NAME: &str = "trigger0";

const DFLT_FREQ: i64 = 100;
const DFLT_NUM_SAMPLE: usize = 100;

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
        let thr = spawn(move || Averager::thread_func(receiver, offset, scale));
        Averager { sender, thr }
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
    let now = SystemTime::now();
    let t = now.duration_since(UNIX_EPOCH).expect("Clock error");
    t.as_secs() as u64 * 1_000_000_000u64 + t.subsec_micros() as u64
}

/////////////////////////////////////////////////////////////////////////////

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
        iio::Context::with_backend(iio::Backend::Network(hostname))
    }
    else if let Some(uri) = matches.value_of("uri") {
        iio::Context::from_uri(uri)
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

    println!("Using device: {}", dev_name);

    // ----- Find the timestamp channel (if any) and a data channel -----

    let mut ts_chan = dev.find_channel("timestamp", false);

    if ts_chan.is_some() {
        println!("Found timestamp channel.");
    }
    else {
        println!("No timestamp channel. Estimating timestamps.");
    }

    let mut sample_chan = dev.find_channel(chan_name, false).unwrap_or_else(|| {
        println!("No '{}' channel on this device", chan_name);
        process::exit(1);
    });

    println!("Using channel: {}", chan_name);

    if sample_chan.type_of() != Some(TypeId::of::<RawSampleType>()) {
        eprintln!("The channel type is different than expected.");
        process::exit(2);
    }

    if let Some(ref mut chan) = ts_chan {
        chan.enable();
    }

    sample_chan.enable();

    // ----- Check for a scale and offset -----

    let offset = sample_chan.attr_read_float("offset").unwrap_or(0.0);
    let scale = sample_chan.attr_read_float("scale").unwrap_or(1.0);

    println!("  Offset: {:.3}, Scale: {:.3}", offset, scale);

    // ----- Set a trigger -----

    let freq = matches
        .value_of("frequency")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(DFLT_FREQ);

    let trig = ctx.find_device(trig_name).unwrap_or_else(|| {
        eprintln!("Couldn't find requested trigger: {}", trig_name);
        process::exit(1);
    });

    // There is no unified way to handle setting sample rates across iio
    // devices. Thus we have to do this handling ourselves.
    // Set the sampling rate on the trigger
    if trig.has_attr("sampling_frequency") {
        if let Err(err) = trig.attr_write_int("sampling_frequency", freq) {
            eprintln!(
                "Can't set sampling rate to {}Hz on {}: '{}'",
                freq,
                trig.name().unwrap(),
                err
            );
        }
    }
    else {
        println!(
            "{} {}",
            "Trigger doesn't have sampling frequency attribute!", "Setting on device instead"
        );

        if dev.has_attr("sampling_frequency") {
            // Set the sampling rate on the device
            if let Err(err) = dev.attr_write_int("sampling_frequency", freq) {
                eprintln!(
                    "Can't set sampling rate to {}Hz on {}: '{}'",
                    freq,
                    dev.name().unwrap(),
                    err
                );
            }
        }
        else {
            eprintln!(
                "{} {}",
                "Can't set sampling frequency! No suitable",
                "attribute found in specified device and trigger..."
            );
            process::exit(2);
        }
    }

    dev.set_trigger(&trig).unwrap_or_else(|err| {
        println!("Error setting the trigger in the device: {}", err);
        process::exit(3);
    });

    // ----- Create a buffer -----

    let n_sample = matches
        .value_of("num_sample")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(DFLT_NUM_SAMPLE);

    let mut buf = dev.create_buffer(n_sample, false).unwrap_or_else(|err| {
        eprintln!("Unable to create buffer: {}", err);
        process::exit(4);
    });

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
        if let Err(err) = buf.refill() {
            eprintln!("Error filling the buffer: {}", err);
            break;
        }

        // Get the timestamp. Use the time of the _last_ sample.

        let ts: u64 = if let Some(ref chan) = ts_chan {
            buf.channel_iter::<u64>(chan)
                .nth(n_sample - 1)
                .unwrap_or_default()
        }
        else {
            timestamp()
        };

        // Extract and convert the raw data from the buffer.
        // This puts the raw samples into host format (fixes "endiness" and
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
}
