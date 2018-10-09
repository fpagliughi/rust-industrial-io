// industrial-io/examples/readraw.rs

extern crate industrial_io as iio;
extern crate schedule_recv;

use std::time::Duration;
use schedule_recv::periodic;

fn main() -> iio::Result<()> {
    let ctx = iio::Context::new().expect("Couldn't open IIO Context");
    let dev = ctx.get_device(0).unwrap();

    let tick = periodic(Duration::from_millis(1000));

    loop {
        tick.recv().unwrap();
        for chan in dev.channels() {
            if let Ok(val) = chan.attr_read_int("raw") {
                print!("  {}", val);
            }
            else {
                print!("  xxxx");
            }
        }
        println!("");
    }
}

