// libiio-sys/src/errors.rs
//
//!
//!

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Nix(::nix::Error);
    }
}

