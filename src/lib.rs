#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

extern crate hex_slice;

pub mod errors {
    use std::string;

    error_chain! {
        foreign_links {
            String(string::FromUtf8Error) #[doc = "Error parsing UTF-8 string"];
        }
    }
}

mod backends;
mod protocol;
pub mod devices;
