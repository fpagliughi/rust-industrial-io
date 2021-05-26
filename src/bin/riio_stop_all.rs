// industrial-io/src/bin/riio_stop_all.rs
//
// Copyright (c) 2019, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

//! Rust application to stop all Industrial I/O devices.
//!
//! It disables the buffer for all devices, and disables all channels for
//! each device as well.
//!
//! This is useful, particularly during development, when a crashed app can
//! leave the devices acquiring data.
//!

extern crate industrial_io as iio;

use std::process;

// --------------------------------------------------------------------------

fn main() {
    let ctx = iio::Context::default().unwrap_or_else(|err| {
        eprintln!("Error getting the IIO Context: {}", err);
        process::exit(1);
    });

    for dev in ctx.devices() {
        /*
        if dev.is_buffer_capable() {
            // The "buffer/enable" attribute isn't documented anywhere,
            // but was discovered in the internals of the libiio C sources.
            if let Err(err) = dev.attr_write_bool("buffer/enable", false) {
                eprintln!("Error disabling buffer: {}", err);
            }
        }
        */

        // We can disable a device by creating a buffer for it
        // and then letting the inner library destroy it cleanly.

        if dev.is_buffer_capable() {
            for mut chan in &mut dev.channels() {
                if chan.is_scan_element() {
                    chan.enable();
                    break;
                }
            }

            let _ = dev.create_buffer(100, false);
        }
    }
}
