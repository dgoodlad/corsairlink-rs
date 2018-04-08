extern crate hidapi;

use hex_slice::AsHex;

use std::fmt;
use errors::*;
use protocol::usbhid as protocol;

const DEFAULT_READ_TIMEOUT: i32 = 1000;

pub struct Device<'a> {
    dev: hidapi::HidDevice<'a>,
    read_timeout: i32,
}

impl<'a> fmt::Debug for Device<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "usbhid backend")
    }
}

impl<'a> Device<'a> {
    pub fn new(dev: hidapi::HidDevice<'a>) -> Device<'a> {
        Device{dev, read_timeout: DEFAULT_READ_TIMEOUT}
    }

    pub fn write_packet<R: protocol::Register, V: protocol::Value<R>>(&self, packet: protocol::TxPacket<R,V>) -> Result<protocol::RxPacket<R, V>> {
        let encoded = packet.encode().unwrap();
        println!("Writing packet: {:x}", encoded.as_hex());
        self.dev.write(&encoded[..])?;

        let mut buf: Vec<u8> = vec![0u8; protocol::PACKET_SIZE];
        self.read(buf.as_mut_slice())?;
        println!("Received response: {:x}", buf.as_hex());
        if buf[0] != encoded[1] {
            self.read(buf.as_mut_slice())?;
            println!("Received response: {:x}", buf.as_hex());
            if buf[0] != encoded[1] {
                self.read(buf.as_mut_slice())?;
                println!("Received response: {:x}", buf.as_hex());
            }
        }
        protocol::RxPacket::decode(packet, &buf[..])
    }

    fn read(&self, buf: &mut [u8]) -> hidapi::HidResult<usize> {
        self.dev.read_timeout(buf, self.read_timeout)
    }
}
