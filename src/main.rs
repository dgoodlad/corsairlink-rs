#![feature(try_from)]

extern crate hidapi;
extern crate hex_slice;

use std::error;
use std::vec::Vec;
use std::fmt;
use hex_slice::AsHex;
use std::convert::TryFrom;

// H110i
const VENDOR_ID: u16 = 0x1b1c;
const PRODUCT_ID: u16 = 0x0c04;

#[derive(Debug)]
enum Error {
    InvalidRegister(u8),
    InvalidOpCode(u8),
    InvalidLEDMode(u8),
    InvalidFanMode(u8),
    ReadError(hidapi::HidError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidRegister(n) => write!(f, "Invalid register byte: 0x{:02x}", n),
            Error::InvalidOpCode(n) => write!(f, "Invalid opcode: 0x{:02x}", n),
            Error::InvalidLEDMode(n) => write!(f, "Invalid LED mode: 0x{:02x}", n),
            Error::InvalidFanMode(n) => write!(f, "Invalid fan mode: 0x{:02x}", n),
            Error::ReadError(ref err) => write!(f, "Error reading from USB device: {}", err)
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidRegister(_) => "Invalid register byte found",
            Error::InvalidOpCode(_) => "Invalid opcode byte found",
            Error::InvalidLEDMode(_) => "Invalid LED mode byte found",
            Error::InvalidFanMode(_) => "Invalid fan mode byte found",
            Error::ReadError(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

#[derive(Debug, Copy, Clone)]
enum Register {
    DeviceId,
    FirmwareId,
    ProductName,
    Status,
    LedSelectCurrent,
    LedCount,
    LedMode,
    LedCurrentColor,
    LedTemperatureColor,
    LedTemperatureMode,
    LedTemperatureModeColors,
    LedCycleColors,
    TempSelectActiveSensor,
    TempCountSensors,
    TempRead,
    TempLimit,
    FanSelect,
    FanCount,
    FanMode,
    FanFixedPWM,
    FanFixedRPM,
    FanReportExtTemp,
    FanReadRPM,
    FanMaxRecordedRPM,
    FanUnderSpeedThreshold,
    FanRPMTable,
    FanTempTable,
}

impl From<Register> for u8 {
    fn from(original: Register) -> u8 {
        match original {
            Register::DeviceId                   => 0x00,
            Register::FirmwareId                 => 0x01,
            Register::ProductName                => 0x02,
            Register::Status                     => 0x03,
            Register::LedSelectCurrent           => 0x04,
            Register::LedCount                   => 0x05,
            Register::LedMode                    => 0x06,
            Register::LedCurrentColor            => 0x07,
            Register::LedTemperatureColor        => 0x08,
            Register::LedTemperatureMode         => 0x09,
            Register::LedTemperatureModeColors   => 0x0A,
            Register::LedCycleColors             => 0x0B,
            Register::TempSelectActiveSensor     => 0x0C,
            Register::TempCountSensors           => 0x0D,
            Register::TempRead                   => 0x0E,
            Register::TempLimit                  => 0x0F,
            Register::FanSelect                  => 0x10,
            Register::FanCount                   => 0x11,
            Register::FanMode                    => 0x12,
            Register::FanFixedPWM                => 0x13,
            Register::FanFixedRPM                => 0x14,
            Register::FanReportExtTemp           => 0x15,
            Register::FanReadRPM                 => 0x16,
            Register::FanMaxRecordedRPM          => 0x17,
            Register::FanUnderSpeedThreshold     => 0x18,
            Register::FanRPMTable                => 0x19,
            Register::FanTempTable               => 0x1A
        }
    }
}

impl TryFrom<u8> for Register {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Register::DeviceId),
            0x01 => Ok(Register::FirmwareId),
            0x02 => Ok(Register::ProductName),
            0x03 => Ok(Register::Status),
            0x04 => Ok(Register::LedSelectCurrent),
            0x05 => Ok(Register::LedCount),
            0x06 => Ok(Register::LedMode),
            0x07 => Ok(Register::LedCurrentColor),
            0x08 => Ok(Register::LedTemperatureColor),
            0x09 => Ok(Register::LedTemperatureMode),
            0x0a => Ok(Register::LedTemperatureModeColors),
            0x0b => Ok(Register::LedCycleColors),
            0x0c => Ok(Register::TempSelectActiveSensor),
            0x0d => Ok(Register::TempCountSensors),
            0x0e => Ok(Register::TempRead),
            0x0f => Ok(Register::TempLimit),
            0x10 => Ok(Register::FanSelect),
            0x11 => Ok(Register::FanCount),
            0x12 => Ok(Register::FanMode),
            0x13 => Ok(Register::FanFixedPWM),
            0x14 => Ok(Register::FanFixedRPM),
            0x15 => Ok(Register::FanReportExtTemp),
            0x16 => Ok(Register::FanReadRPM),
            0x17 => Ok(Register::FanMaxRecordedRPM),
            0x18 => Ok(Register::FanUnderSpeedThreshold),
            0x19 => Ok(Register::FanRPMTable),
            0x1a => Ok(Register::FanTempTable),
            n => Err(Error::InvalidRegister(n))
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}(0x{:02x})", self, u8::from(*self))
    }
}

#[derive(Debug, Copy, Clone)]
enum OpCode {
    WriteOneByte,
    ReadOneByte,
    WriteTwoBytes,
    ReadTwoBytes,
    WriteManyBytes,
    ReadManyBytes,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}(0x{:02x})", self, u8::from(*self))
    }
}

impl From<OpCode> for u8 {
    fn from(op_code: OpCode) -> u8 {
        match op_code {
            OpCode::WriteOneByte   => 0x06,
            OpCode::ReadOneByte    => 0x07,
            OpCode::WriteTwoBytes  => 0x08,
            OpCode::ReadTwoBytes   => 0x09,
            OpCode::WriteManyBytes => 0x0a,
            OpCode::ReadManyBytes  => 0x0b
        }
    }
}

impl TryFrom<u8> for OpCode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x06 => Ok(OpCode::WriteOneByte),
            0x07 => Ok(OpCode::ReadOneByte),
            0x08 => Ok(OpCode::WriteTwoBytes),
            0x09 => Ok(OpCode::ReadTwoBytes),
            0x0a => Ok(OpCode::WriteManyBytes),
            0x0b => Ok(OpCode::ReadManyBytes),
            n => Err(Error::InvalidOpCode(n))
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum LedMode {
    StaticColor = 0x00,
    TwoColorCycle = 0x40,
    FourColorCycle = 0x80,
    TemperatureColor = 0xC0
}

impl fmt::Display for LedMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:02x}", *self as u8)
    }
}

impl From<LedMode> for u8 {
    fn from(led_mode: LedMode) -> u8 {
        match led_mode {
            LedMode::StaticColor      => 0x00,
            LedMode::TwoColorCycle    => 0x40,
            LedMode::FourColorCycle   => 0x80,
            LedMode::TemperatureColor => 0xC0
        }
    }
}

impl TryFrom<u8> for LedMode {
    type Error = Error;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(LedMode::StaticColor),
            0x40 => Ok(LedMode::TwoColorCycle),
            0x80 => Ok(LedMode::FourColorCycle),
            0xC0 => Ok(LedMode::TemperatureColor),
            n => Err(Error::InvalidLEDMode(n))
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum FanMode {
    FixedPWM = 0x02,
    FixedRPM = 0x04,
    Default = 0x06,
    Quiet = 0x08,
    Balanced = 0x0A,
    Performance = 0x0C,
    Custom = 0x0E,
}

impl fmt::Display for FanMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{:02x}", *self as u8)
    }
}

impl From<FanMode> for u8 {
    fn from(fan_mode: FanMode) -> u8 {
        match fan_mode {
            FanMode::FixedPWM    => 0x02,
            FanMode::FixedRPM    => 0x04,
            FanMode::Default     => 0x06,
            FanMode::Quiet       => 0x08,
            FanMode::Balanced    => 0x0A,
            FanMode::Performance => 0x0C,
            FanMode::Custom      => 0x0E
        }
    }
}

impl TryFrom<u8> for FanMode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x02 => Ok(FanMode::FixedPWM),
            0x04 => Ok(FanMode::FixedRPM),
            0x06 => Ok(FanMode::Default),
            0x08 => Ok(FanMode::Quiet),
            0x0A => Ok(FanMode::Balanced),
            0x0C => Ok(FanMode::Performance),
            0x0E => Ok(FanMode::Custom),
            n => Err(Error::InvalidFanMode(n))
        }
    }
}

fn main() {
    let api = hidapi::HidApi::new().expect("Failed to create API instance");

    let mut cooler = Cooler::open(&api, VENDOR_ID, PRODUCT_ID).expect("BOOM");

    let op = CoolerOp {
        op_code: OpCode::ReadManyBytes,
        register: Register::ProductName,
        data: vec![32u8], // read 32-byte product name
    };

    println!("{}", op);
    cooler.write_op(&op);

    let op = match cooler.read_response() {
        Ok(op) => op,
        Err(e) => panic!("{}", e)
    };
}

#[derive(Debug)]
struct CoolerOp {
    op_code: OpCode,
    register: Register,
    data: Vec<u8>,
}

impl CoolerOp {
    fn len(&self) -> u8 {
        return self.data.len() as u8 + 2;
    }

    fn to_vec(&self) -> Vec<u8> {
        let mut data = self.data.clone();
        let mut buf = vec![u8::from(self.op_code), u8::from(self.register)];
        buf.append(&mut data);
        return buf;
    }

    fn from_vec(v: Vec<u8>) -> Self {
        return Self {
            op_code: OpCode::try_from(v[0] as u8).unwrap(),
            register: Register::try_from(v[1] as u8).unwrap(),
            data: v[2..].to_vec()
        }
    }
}

impl fmt::Display for CoolerOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {:x}", self.op_code, self.register, self.data.as_hex())
    }
}

struct Cooler<'a> {
    device: hidapi::HidDevice<'a>,
    last_id: u8,
}

impl<'a> Cooler<'a> {
    fn open(api: &'a hidapi::HidApi, vendor_id: u16, product_id: u16) -> Result<Self, hidapi::HidError> {
        match api.open(vendor_id, product_id) {
            Ok(device) => Ok(Cooler {device: device, last_id: 9}),
            Err(e) => Err(e)
        }
    }

    fn write_op(&mut self, op: &CoolerOp) {
        let id = self.last_id + 1;
        let len = 1 + op.len();
        let data = op.to_vec();

        self.last_id = id;

        let mut buf = vec![len as u8, id as u8];
        buf.extend(&data);
        match self.device.write(&buf[..]) {
            Ok(size) => println!("Wrote {:?} bytes: {:?}", size, buf),
            Err(e) => println!("Error writing op: {:?}", e)
        }
    }

    fn read_response(&self) -> Result<CoolerOp, Error> {
        let mut buf = [0u8; 64];
        return match self.device.read_timeout(&mut buf[..], 1000) {
            Ok(_) => Ok(CoolerOp::from_vec(buf.to_vec())),
            Err(e) => Err(Error::ReadError(e))
        }
    }
}