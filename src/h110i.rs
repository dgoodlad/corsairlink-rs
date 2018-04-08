#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum Opcode {
    WriteByte = 0x06,
    ReadByte = 0x07,
    WriteWord = 0x08,
    ReadWord = 0x09,
    WriteBlock = 0x0a,
    ReadBlock = 0x0b,
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

impl Register {
    fn len(&self) -> usize {
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

#[derive(Debug)]
pub enum RegisterValue {
    DeviceId(u8),
    FirmwareVersion(u8,u8),
    ProductName(String),
    Status(u8),
    LedSelect(u8),

    TempSensorSelect(u8),
    TempSensorCount(u8),
    TempSensorValue(u8,u8),
    TempSensorLimit(u8,u8),
}

type DecodeError = &'static str;

impl RegisterValue {
    fn decode(register: Register, data: &[u8]) -> Result<RegisterValue, DecodeError> {
        match register {
            Register::DeviceId => Ok(RegisterValue::DeviceId(data[0])),
            Register::FirmwareVersion => Ok(RegisterValue::FirmwareVersion(data[0], data[1])),
            Register::ProductName => {
                let s = match data[1..].iter().position(|x| { *x == 0 }) {
                    Some(n) => String::from_utf8(data[1..n+1].to_vec()),
                    None => return Err("No null byte found while parsing product name string"),
                };
                match s {
                    Ok(string) => Ok(RegisterValue::ProductName(string)),
                    Err(_) => Err("Error parsing UTF-8 string for product name"),
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

    fn encode_into(&self, buf: &mut [u8]) -> Option<usize> {
        match self {
            &RegisterValue::LedSelect(led) => { buf[0] = led; Some(1) },
            &RegisterValue::TempSensorSelect(sensor) => { buf[0] = sensor; Some(1) },
            &RegisterValue::TempSensorLimit(lb,hb) => { buf[0] = lb; buf[1] = hb; Some(2) },
            _ => None
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Read(Register),
    Write(Register, RegisterValue),
}

impl Command {
    fn opcode(&self) -> Opcode {
        match self {
            &Command::Read(ref register) => match register.len() {
                1 => Opcode::ReadByte,
                2 => Opcode::ReadWord,
                _ => Opcode::ReadBlock,
            },
            &Command::Write(ref register, _) => match register.len() {
                1 => Opcode::WriteByte,
                2 => Opcode::WriteWord,
                _ => Opcode::WriteBlock,
            }
        }
    }

    fn register(&self) -> Register {
        match self {
            &Command::Read(register) => register,
            &Command::Write(register, _) => register,
        }
    }

    fn encode_into(&self, buf: &mut [u8]) -> Option<usize> {
        buf[0] = self.opcode() as u8;
        buf[1] = self.register() as u8;

        match self {
            &Command::Write(register, ref value) => {
                value.encode_into(buf[2..].as_mut());
                Some(2 + register.len())
            },
            &Command::Read(_) => Some(2)
        }
    }

    fn len(&self) -> usize {
        match self {
            &Command::Read(_) => 2,
            &Command::Write(register, _) => 2 + register.len(),
        }
    }
}

const PACKET_SIZE: usize = 64;
const FIRST_COMMAND_ID: u8 = 20;

pub struct TxPacket {
    first_command_id: u8,
    commands: Vec<Command>,
}

impl TxPacket {
    pub fn new(first_command_id: u8, commands: Vec<Command>) -> TxPacket {
        TxPacket { first_command_id, commands }
    }
}

impl TxPacket {
    pub fn encode(&self) -> Option<Vec<u8>> {
        let len = self.len();
        let mut buf: Vec<u8> = vec![0; len];
        buf[0] = len as u8;

        let mut i = 1;
        let mut command_id = self.first_command_id;

        for c in self.commands.iter() {
            buf[i] = command_id;
            i += 1;
            match buf.get_mut(i .. i + c.len()) {
                Some(slice) => c.encode_into(slice),
                None => return None
            };
            i += c.len();
            command_id += 1;
        }

        Some(buf)
    }

    pub fn len(&self) -> usize {
        self.commands.iter().fold(1, |sum, c| { sum + c.len() + 1 })
    }
}

#[derive(Debug)]
pub enum RxCommand {
    Read(Register, RegisterValue),
    Write(Register),
}

impl RxCommand {
    pub fn decode_read(register: Register, data: &[u8]) -> Result<RxCommand, DecodeError> {
        Ok(RxCommand::Read(register, RegisterValue::decode(register, data)?))
    }

    pub fn len(&self) -> usize {
        match self {
            &RxCommand::Read(register, _) => match register.len() {
                1 => 2,
                2 => 3,
                len @ _ => len + 2,
            },
            &RxCommand::Write(_) => 1
        }
    }
}

#[derive(Debug)]
pub struct RxPacket(Vec<RxCommand>);

impl RxPacket {
    pub fn decode(tx_packet: TxPacket, data: &[u8]) -> Option<RxPacket> {
        let mut rxpacket = RxPacket(Vec::new());

        let mut command_id = tx_packet.first_command_id;
        let mut i = 0;
        for c in tx_packet.commands.iter() {
            println!("Decoding for {:?}", c);
            if data[i] != command_id {
                println!("Bad command ID {}", data[i]);
                return None;
            }

            let rxcommand = match c {
                &Command::Read(register) => {
                    match RxCommand::decode_read(register, &data[i + 2 .. ]) {
                        Ok(decoded) => decoded,
                        Err(_) => return None,
                    }
                },
                &Command::Write(register, _) => RxCommand::Write(register),
            };

            command_id += 1;
            i += rxcommand.len() + 1;
            rxpacket.0.push(rxcommand);
        }

        Some(rxpacket)
    }
}
