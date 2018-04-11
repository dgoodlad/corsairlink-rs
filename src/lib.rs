#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

extern crate byteorder;
extern crate hex_slice;

extern crate libusb;

pub mod errors {
    use std::string;
    use libusb;

    error_chain! {
        foreign_links {
            String(string::FromUtf8Error) #[doc = "Error parsing UTF-8 string"];
            Libusb(libusb::Error) #[doc = "Error from libusb"];
        }
    }
}

mod backends;
mod protocol;
pub mod devices;
