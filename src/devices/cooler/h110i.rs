pub use backends::usbhid as backend;
use protocol::usbhid;

pub const VENDOR_ID: u16 = 0x1b1c;
pub const PRODUCT_ID: u16 = 0x0c04;

pub use protocol::usbhid::Command;
pub use protocol::usbhid::TxPacket;
pub use protocol::usbhid::RxPacket;

use errors::*;

#[derive(Debug)]
pub struct Device<'a> {
    backend: backend::Device<'a>,

    device_id: u8,
    firmware_version: String,
    product_name: String,

    fan_speeds: Vec<u16>,
    temperatures: Vec<f64>,
}

impl<'a> Device<'a> {
    pub fn new(backend: backend::Device) -> Device {
        Device {
            backend,

            device_id: 0,
            firmware_version: "".to_string(),
            product_name: "".to_string(),

            fan_speeds: vec![],
            temperatures: vec![],
        }
    }

    pub fn get_metadata(&mut self) -> Result<()> {
        let commands: Vec<Command<Register, RegisterValue>> = vec![
            Command::Read(Register::DeviceId),
            Command::Read(Register::FirmwareVersion),
            Command::Read(Register::ProductName),
        ];

        let txpacket = TxPacket::new(20, commands);
        let rxpacket = self.backend.write_packet(txpacket)?;

        for value in rxpacket.read_values() {
            match value {
                RegisterValue::DeviceId(device_id) => self.device_id = device_id,
                RegisterValue::FirmwareVersion(s) => self.firmware_version = s,
                RegisterValue::ProductName(s) => self.product_name = s,
                _ => (),
            }
        };

        Ok(())
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Register {
    DeviceId = 0x00,
    FirmwareVersion = 0x01,
    ProductName = 0x02,
    Status = 0x03,
    LedSelect = 0x04,

    TempSensorSelect = 0x0c,
    TempSensorCount = 0x0d,
    TempSensorValue = 0x0e,
    TempSensorLimit = 0x0f,
}

impl Into<u8> for Register {
    fn into(self) -> u8 { self as u8 }
}

impl usbhid::Register for Register {
    fn size(&self) -> usize {
        match self {
            &Register::DeviceId => 1,
            &Register::FirmwareVersion => 2,
            &Register::ProductName => 8,
            &Register::Status => 1,
            &Register::LedSelect => 1,

            &Register::TempSensorSelect => 1,
            &Register::TempSensorCount => 1,
            &Register::TempSensorValue => 2,
            &Register::TempSensorLimit => 2,
        }
    }
}

#[derive(Clone, Debug)]
pub enum RegisterValue {
    DeviceId(u8),
    FirmwareVersion(String),
    ProductName(String),
    Status(u8),
    LedSelect(u8),

    TempSensorSelect(u8),
    TempSensorCount(u8),
    TempSensorValue(u8,u8),
    TempSensorLimit(u8,u8),
}

impl RegisterValue {
    fn decode_firmware_version(lb: u8, hb: u8) -> String {
        format!("{:02x}.{:x}.{:x}", hb, lb & 0xf0 >> 4, lb & 0x0f)
    }
}

impl usbhid::Value<Register> for RegisterValue {
    type DecodeError = &'static str;

    fn decode(register: Register, data: &[u8]) -> Result<Self> {
        match register {
            Register::DeviceId => Ok(RegisterValue::DeviceId(data[0])),
            Register::FirmwareVersion => Ok(RegisterValue::FirmwareVersion(
                RegisterValue::decode_firmware_version(data[0], data[1]))),
            Register::ProductName => {
                match data[1..].iter().position(|x| { *x == 0 }) {
                    Some(n) => Ok(RegisterValue::ProductName(
                        String::from_utf8(data[1..n+1].to_vec())?)),
                    None => return Err("No null byte found while parsing product name string".into()),
                }
            },
            Register::Status => Ok(RegisterValue::Status(data[0])),
            Register::LedSelect => Ok(RegisterValue::LedSelect(data[0])),

            Register::TempSensorSelect => Ok(RegisterValue::TempSensorSelect(data[0])),
            Register::TempSensorCount => Ok(RegisterValue::TempSensorCount(data[0])),
            Register::TempSensorValue => Ok(RegisterValue::TempSensorValue(data[0], data[1])),
            Register::TempSensorLimit => Ok(RegisterValue::TempSensorLimit(data[0], data[1])),
        }
    }

    fn encode(&self, buf: &mut [u8]) -> Option<usize> {
        match self {
            &RegisterValue::LedSelect(led) => { buf[0] = led; Some(1) },
            &RegisterValue::TempSensorSelect(sensor) => { buf[0] = sensor; Some(1) },
            &RegisterValue::TempSensorLimit(lb,hb) => { buf[0] = lb; buf[1] = hb; Some(2) },
            _ => None
        }
    }
}
