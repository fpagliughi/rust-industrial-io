// industrial-io/examples/riio_detect.rs
//
// Simple Rust IIO example to list the devices found in the specified context.
//
// Note that, if no context is requested at the command line, this will create
// a network context if the IIOD_REMOTE environment variable is set, otherwise
// it will create a local context. See Context::new().
//
// Copyright (c) 2023, Frank Pagliughi, All Rights Reserved
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

#[cfg(feature = "libiio_v0_19")]
fn main() {
    println!("Scan Contexts not supported before libiio v0.20");
}

#[cfg(not(feature = "libiio_v0_19"))]
fn main() {
    use industrial_io as iio;
    use std::process;

    for backend in &["local", "ip", "usb"] {
        let scan_ctx = iio::ScanContext::new(backend).unwrap_or_else(|err| {
            eprintln!("Can't create scan context: {}", err);
            process::exit(1);
        });

        let n = scan_ctx.len();
        if n == 0 {
            continue;
        }

        println!("{}: [{}]", backend, n);
        for ctx in scan_ctx.iter() {
            println!("  {}: {}", ctx.0, ctx.1);
        }
    }
}
