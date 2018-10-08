// industrial-io/src/lib.rs
//
//!
//!

extern crate libiio_sys as ffi;

pub use context::*;
pub use device::*;
pub use channel::*;
pub use buffer::*;
pub use errors::*;

pub mod context;
pub mod device;
pub mod channel;
pub mod buffer;
pub mod errors;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
