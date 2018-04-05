use phy;

#[repr(u8)]
enum Mode {
    WriteByte = 0x06,
    ReadByte = 0x07,
    WriteWord = 0x08,
    ReadWord = 0x09,
    WriteBlock = 0x0a,
    ReadBlock = 0x0b,
}

#[repr(u8)]
pub enum Register {
    DeviceId = 0x00,
    FirmwareVersion = 0x01,
    ProductName = 0x02,
    Status = 0x03,
    LedSelect = 0x04,
}


impl Register {
    fn len(&self) -> usize {
        match self {
            &Register::DeviceId => 1,
            &Register::FirmwareVersion => 2,
            &Register::ProductName => 8,
            &Register::Status => 1,
            &Register::LedSelect => 1,
        }
    }
}

pub enum RegisterValue {
    DeviceId(u8),
    FirmwareVersion(u8,u8),
    ProductName(String),
    Status(u8),
    LedSelect(u8),
}

impl RegisterValue {
    fn encode_into(&self, buf: &mut [u8]) -> Option<usize> {
        match self {
            &RegisterValue::LedSelect(led) => { buf[0] = led; Some(1) },
            _ => None
        }
    }
}

pub enum Command {
    Read(Register),
    Write(Register, RegisterValue),
}

impl phy::Encodable for Command {
    fn encode_into(&self, buf: &mut [u8]) -> Option<usize> {
        buf[0] = match self {
            &Command::Read(ref register) => match register.len() {
                1 => Mode::ReadByte,
                2 => Mode::ReadWord,
                _ => Mode::ReadBlock,
            },
            &Command::Write(ref register, _) => match register.len() {
                1 => Mode::WriteByte,
                2 => Mode::WriteWord,
                _ => Mode::WriteBlock,
            }
        } as u8;

        match self {
            &Command::Write(ref register, ref value) => {
                value.encode_into(buf[1..].as_mut());
                Some(1 + register.len())
            },
            &Command::Read(_) => Some(1)
        }
    }

    fn len(&self) -> usize {
        match self {
            &Command::Read(ref register) => register.len(),
            &Command::Write(ref register, _) => register.len(),
        }
    }
}
