// libiio-sys/build.rs
//
// The builder for the Linux Industrial I/O wrapper crate.
//

fn main() {
    println!("cargo:rustc-link-lib=iio");
}


