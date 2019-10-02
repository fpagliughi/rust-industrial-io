// industrial-io/src/bin/iio_info_rs.rs
//
// Copyright (c) 2018, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//! Rust application to gather information about Industrial I/O devices.
//!

extern crate industrial_io as iio;

use std::process;

fn main() -> iio::Result<()> {

    let ctx = iio::Context::new().unwrap_or_else(|err| {
        eprintln!("Error getting the IIO Context: {}", err);
        process::exit(1);
    });

    println!("Description: {}", ctx.description());
    println!("IIO context has {} devices:", ctx.num_devices());

    for dev in ctx.devices() {
        //assert_eq(ctx, dev.context());
        println!("\t{}: {}", dev.id().unwrap_or_default(),
                 dev.name().unwrap_or_else(|| "<unknown>".to_string()));
        println!("\t\t{} channels found:", dev.num_channels());

        for chan in dev.channels() {
            println!("\t\t\t{}", chan.id().unwrap_or_default());
            println!("\t\t\t{} channel-specific attributes found:", chan.num_attrs());
            for attr in chan.attrs() {
                print!("\t\t\t\t'{}' value: ", attr);
                if let Ok(val) = chan.attr_read_float(&attr) {
                    println!("{}", val);
                }
                else if let Ok(val) = chan.attr_read_int(&attr) {
                    println!("{}", val);
                }
                else if let Ok(val) = chan.attr_read_bool(&attr) {
                    println!("{}", val);
                }
                else {
                    println!();
                }
            }
        }
    }
    Ok(())
}

