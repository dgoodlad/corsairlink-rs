use hex_slice::AsHex;

use std::fmt;
use std::time::Duration;
use errors::*;
use protocol::usbhid as protocol;
use libusb;

const DEFAULT_READ_TIMEOUT: u64 = 1000;
const DEFAULT_WRITE_TIMEOUT: u64 = 1000;

const HID_SET_REPORT: u8 = 0x09;
const HID_REPORT_TYPE_OUTPUT: u16 = 0x02;
const HID_REPORT_NUMBER: u16 = 0x00;
const INTERFACE_NUMBER: u8 = 0;
const INTERRUPT_IN_ENDPOINT: u8 = 0x81;

pub struct Device<'a> {
    dev: libusb::DeviceHandle<'a>,
    read_timeout: Duration,
    write_timeout: Duration,
}

impl<'a> fmt::Debug for Device<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "usbhid backend")
    }
}

impl<'a> Device<'a> {
    pub fn open(context: &'a libusb::Context, vendor_id: u16, product_id: u16) -> Result<Device<'a>> {
        for mut device in context.devices().unwrap().iter() {
            let device_desc = device.device_descriptor().unwrap();

            if device_desc.vendor_id() == vendor_id && device_desc.product_id() == product_id {
                let mut handle = device.open().unwrap();
                if handle.kernel_driver_active(INTERFACE_NUMBER)? {
                    handle.detach_kernel_driver(INTERFACE_NUMBER)?;
                }
                handle.claim_interface(INTERFACE_NUMBER)?;

                return Ok(Device {
                    dev: handle,
                    read_timeout: Duration::from_millis(DEFAULT_READ_TIMEOUT),
                    write_timeout: Duration::from_millis(DEFAULT_WRITE_TIMEOUT),
                })
            }
        };

        Err("No device found".into())
    }

    fn write(&self, data: &[u8]) -> Result<usize> {
        self.dev.write_control(
            libusb::request_type(libusb::Direction::Out, libusb::RequestType::Class, libusb::Recipient::Interface),
            HID_SET_REPORT, // 0x09
            HID_REPORT_TYPE_OUTPUT << 8 | HID_REPORT_NUMBER,
            INTERFACE_NUMBER as u16,
            data,
            self.write_timeout,
        ).chain_err(|| "Error writing to USB device")
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        self.dev.read_interrupt(
            INTERRUPT_IN_ENDPOINT,
            buf,
            self.read_timeout
        ).chain_err(|| "Error reading from USB device")
    }

    pub fn write_packet<R: protocol::Register, V: protocol::Value<R>>(&self, packet: protocol::TxPacket<R,V>) -> Result<protocol::RxPacket<R, V>> {
        let encoded = packet.encode().unwrap();
        println!("Writing packet: {:x}", encoded.as_hex());
        self.write(&encoded[..])?;

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
}
