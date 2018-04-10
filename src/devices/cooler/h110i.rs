use std::fmt;
use errors::*;

pub use backends::usbhid as backend;
use libusb;
use protocol::usbhid;
use protocol::usbhid::Command;
use protocol::usbhid::TxPacket;

use byteorder::{ByteOrder, LittleEndian};

pub const VENDOR_ID: u16 = 0x1b1c;
pub const PRODUCT_ID: u16 = 0x0c04;

#[derive(Debug, Copy, Clone)]
pub struct Temperature(u16);

impl Temperature {
    fn degrees_c(&self) -> f64 {
        self.0 as f64 / 256.0
    }
}

impl From<Temperature> for u16 {
    fn from(t: Temperature) -> u16 {
        t.0
    }
}

impl From<Temperature> for f64 {
    fn from(t: Temperature) -> f64 {
        t.degrees_c()
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}°C", self.degrees_c())
    }
}

#[derive(Debug)]
pub struct Device<'a> {
    backend: backend::Device<'a>,

    device_id: u8,
    firmware_version: String,
    product_name: String,

    led_count: u8,
    temp_sensor_count: u8,
    fan_count: u8,

    pub led_modes: Vec<LedMode>,
    pub temperatures: Vec<Temperature>,
    pub fan_speeds: Vec<u16>,
}

impl<'a> Device<'a> {
    pub fn open(context: &'a libusb::Context) -> Result<Device<'a>> {
        let dev = backend::Device::open(context, VENDOR_ID, PRODUCT_ID)?;
        Ok(Self::new(dev))
    }

    pub fn new(backend: backend::Device) -> Device {
        Device {
            backend,

            device_id: 0,
            firmware_version: "".to_string(),
            product_name: "".to_string(),

            led_count: 0,
            temp_sensor_count: 0,
            fan_count: 0,

            led_modes: vec![],
            temperatures: vec![],
            fan_speeds: vec![],
        }
    }

    pub fn get_metadata(&mut self) -> Result<()> {
        let commands: Vec<Command<Register, RegisterValue>> = vec![
            Command::Read(Register::DeviceId),
            Command::Read(Register::FirmwareVersion),
            Command::Read(Register::ProductName),
            Command::Read(Register::LedCount),
            Command::Read(Register::TempSensorCount),
            Command::Read(Register::FanCount),
        ];

        let txpacket = TxPacket::new(20, commands);
        let rxpacket = self.backend.write_packet(txpacket)?;

        for value in rxpacket.read_values() {
            match value {
                RegisterValue::DeviceId(device_id) => self.device_id = device_id,
                RegisterValue::FirmwareVersion(s) => self.firmware_version = s,
                RegisterValue::ProductName(s) => self.product_name = s,
                RegisterValue::LedCount(i) => self.led_count = i,
                RegisterValue::TempSensorCount(i) => self.temp_sensor_count = i,
                RegisterValue::FanCount(i) => self.fan_count = i,
                _ => (),
            }
        };

        Ok(())
    }

    pub fn poll_temperatures(&mut self) -> Result<()> {
        let mut commands: Vec<Command<Register, RegisterValue>> = Vec::new();
        for i in 0..self.temp_sensor_count {
            commands.push(Command::Write(Register::TempSensorSelect, RegisterValue::TempSensorSelect(i as u8)));
            commands.push(Command::Read(Register::TempSensorValue));
        }

        let txpacket = TxPacket::new(20, commands);
        let rxpacket = self.backend.write_packet(txpacket)?;

        for value in rxpacket.read_values() {
            match value {
                RegisterValue::TempSensorValue(lb, hb) => self.temperatures.push(Temperature(LittleEndian::read_u16(&[lb, hb]))),
                _ => (),
            };
        }

        Ok(())
    }

    pub fn poll_led_modes(&mut self) -> Result<()> {
        for i in 0..self.led_count {
            let mut commands: Vec<Command<Register, RegisterValue>> = Vec::new();
            commands.push(Command::Write(Register::LedSelect, RegisterValue::FanSelect(i as u8)));
            commands.push(Command::Read(Register::LedMode));

            let txpacket = TxPacket::new(20, commands);
            let rxpacket = self.backend.write_packet(txpacket)?;

            for value in rxpacket.read_values() {
                match value {
                    RegisterValue::LedMode(mode) => self.led_modes.push(mode),
                    _ => (),
                }
            }
        }

        Ok(())
    }

    pub fn poll_fans(&mut self) -> Result<()> {
        for i in 0..self.fan_count {
            let mut commands: Vec<Command<Register, RegisterValue>> = Vec::new();
            commands.push(Command::Write(Register::FanSelect, RegisterValue::FanSelect(i as u8)));
            commands.push(Command::Read(Register::FanRPM));

            let txpacket = TxPacket::new(20, commands);
            let rxpacket = self.backend.write_packet(txpacket)?;

            for value in rxpacket.read_values() {
                match value {
                    RegisterValue::FanRPM(rpm) => self.fan_speeds.push(rpm),
                    _ => (),
                };
            }
        }

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
    LedCount = 0x05,
    LedMode = 0x06,
    LedColor = 0x07,
    LedTemperatureColor = 0x08,
    LedTemperatureModeTemps = 0x09,
    LedTemperatureModeColors = 0x0a,
    LedCycleColors = 0x0b,

    TempSensorSelect = 0x0c,
    TempSensorCount = 0x0d,
    TempSensorValue = 0x0e,
    TempSensorLimit = 0x0f,

    FanSelect = 0x10,
    FanCount = 0x11,
    FanMode = 0x012,
    FanFixedPWM = 0x13,
    FanFixedRPM = 0x14,
    FanReportExtTemp = 0x15,
    FanRPM = 0x16,
    FanMaxRecordedRPM = 0x17,
    FanUnderSpeedThreshold = 0x18,
    FanRPMTable = 0x19,
    FanTempTable = 0x1a,
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
            &Register::LedCount => 1,
            &Register::LedMode => 1,
            &Register::LedColor => 3,
            &Register::LedTemperatureColor => 2,
            &Register::LedTemperatureModeTemps => 6,
            &Register::LedTemperatureModeColors => 9,
            &Register::LedCycleColors => 12,

            &Register::TempSensorSelect => 1,
            &Register::TempSensorCount => 1,
            &Register::TempSensorValue => 2,
            &Register::TempSensorLimit => 2,

            &Register::FanSelect => 1,
            &Register::FanMode => 1,
            &Register::FanCount => 1,
            &Register::FanFixedPWM => 1,
            &Register::FanFixedRPM => 2,
            &Register::FanReportExtTemp => 2,
            &Register::FanRPM => 2,
            &Register::FanMaxRecordedRPM => 2,
            &Register::FanUnderSpeedThreshold => 2,
            &Register::FanRPMTable => 10,
            &Register::FanTempTable => 10,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum TempChannel {
    InternalSensor = 0x0,
    Manual = 0x7,
}

impl TempChannel {
    pub fn decode(data: u8) -> Result<TempChannel> {
        match data {
            0x0 => Ok(TempChannel::InternalSensor),
            0x7 => Ok(TempChannel::Manual),
            _ => Err("Invalid temperature channel for LED mode".into())
        }
    }
}

#[derive(Clone, Debug)]
pub enum LedMode {
    Static,
    TwoColorCycle(u8),
    FourColorCycle(u8),
    Temperature(TempChannel),
}

impl LedMode {
    fn decode(data: u8) -> Result<LedMode> {
        match data & 0xf0 {
            0x00 => Ok(LedMode::Static),
            0x40 => Ok(LedMode::TwoColorCycle(data & 0x0f)),
            0x80 => Ok(LedMode::FourColorCycle(data & 0x0f)),
            0xC0 => Ok(LedMode::Temperature(TempChannel::decode(data & 0x0f)?)),
            _ => Err("Invalid LED mode byte".into())
        }
    }

    fn encode(&self) -> u8 {
        match self {
            &LedMode::Static => 0x00,
            &LedMode::TwoColorCycle(speed) => 0x40 | (speed & 0x0f),
            &LedMode::FourColorCycle(speed) => 0x80 | (speed & 0x0f),
            &LedMode::Temperature(channel) => 0xC0 | channel as u8,
        }
    }

    fn cycle_speed(&self) -> Result<u8> {
        match self {
            &LedMode::TwoColorCycle(speed) | &LedMode::FourColorCycle(speed) => Ok(speed),
            _ => Err("Cycle speed is not defined for this LED mode".into()),
        }
    }

    fn temp_channel(&self) -> Result<TempChannel> {
        match self {
            &LedMode::Temperature(channel) => Ok(channel),
            _ => Err("Temperature channel is not defined for this LED mode".into()),
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
    LedCount(u8),
    LedMode(LedMode),

    TempSensorSelect(u8),
    TempSensorCount(u8),
    TempSensorValue(u8,u8),
    TempSensorLimit(u8,u8),

    FanSelect(u8),
    FanCount(u8),
    FanRPM(u16),
}

impl RegisterValue {
    fn decode_firmware_version(lb: u8, hb: u8) -> String {
        format!("{:x}.{:x}.{:02x}", (hb & 0xf0) >> 4, hb & 0x0f, lb)
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
            Register::LedCount => Ok(RegisterValue::LedCount(data[0])),
            Register::LedMode => Ok(RegisterValue::LedMode(LedMode::decode(data[0])?)),

            Register::TempSensorSelect => Ok(RegisterValue::TempSensorSelect(data[0])),
            Register::TempSensorCount => Ok(RegisterValue::TempSensorCount(data[0])),
            Register::TempSensorValue => Ok(RegisterValue::TempSensorValue(data[0], data[1])),
            Register::TempSensorLimit => Ok(RegisterValue::TempSensorLimit(data[0], data[1])),

            Register::FanSelect => Ok(RegisterValue::FanSelect(data[0])),
            Register::FanCount => Ok(RegisterValue::FanCount(data[0])),
            Register::FanRPM => Ok(RegisterValue::FanRPM(LittleEndian::read_u16(&data[0..2]))),

            _ => Err("Unhandled register".into()),
        }
    }

    fn encode(&self, buf: &mut [u8]) -> Option<usize> {
        match self {
            &RegisterValue::LedSelect(led) => { buf[0] = led; Some(1) },
            &RegisterValue::TempSensorSelect(sensor) => { buf[0] = sensor; Some(1) },
            &RegisterValue::TempSensorLimit(lb,hb) => { buf[0] = lb; buf[1] = hb; Some(2) },
            &RegisterValue::FanSelect(fan) => { buf[0] = fan; Some(1) },
            _ => None
        }
    }
}
