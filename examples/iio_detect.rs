// industrial-io/examples/iio_detect.rs
//
// Simple Rust IIO example. This lists the devices found in the default context.
//
// Copyright (c) 2018, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

extern crate industrial_io as iio;
use std::process;

fn main() {
    let ctx = iio::Context::new().unwrap_or_else(|_err| {
        println!("Couldn't open default IIO context");
        process::exit(1);
    });

    let mut trigs = Vec::new();

    if ctx.num_devices() == 0 {
        println!("No devices in the default IIO context");
    } else {
        println!("IIO Devices:");
        for dev in ctx.devices() {
            if dev.is_trigger() {
                if let Some(id) = dev.id() {
                    trigs.push(id);
                }
            } else {
                print!("  {} ", dev.id().unwrap_or_default());
                print!("[{}]", dev.name().unwrap_or_default());
                println!(": {} channel(s)", dev.num_channels());
            }
        }

        if !trigs.is_empty() {
            println!("\nTriggers Devices:");
            for s in trigs {
                println!("  {}", s);
            }
        }
    }
}
