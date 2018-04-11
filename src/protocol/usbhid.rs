//! # Corsair Link over USB HID
//!
//! The USB HID protocol used by some Corsair Link devices is similar to SMBus,
//! but operates using raw USB HID reports. The host computer writes 64-byte
//! packets to report number 0x00, and is expected to follow each write by reading
//! that same report number, resulting in a 64-byte response.
//!
//! All values are little-endian.
//!
//! Transmitted packets are structured as follows:
//!
//!     LEN <CommandID> <Command> <Command..?> <Zero Padding>
//!
//! The first byte is the total length in bytes of the command data contained
//! in the packet (not including the first len byte). The packet is then
//! zero-padded to 64 bytes. Each command is prefixed by an identifier in the
//! range 20..255 inclusive, which is then used to identify responses in the
//! reply packet.
//!
//! Each command is an SMBus-style command, consisting of an opcode, a register,
//! and optionally some data. The opcodes either read or write from a given
//! register. The correct opcode should be chosen for a given register, based on
//! the length of data that register stores. There are opcodes for operating on
//! single bytes, words (two bytes), or on arbitrary-sized data ("blocks"). When
//! operating on a block register, a LEN byte is the first byte of the command
//! data. For example:
//!
//!    [0x07 0x00]
//!      |    \----- Register 0x00: Device ID
//!      \---------- Opcode 0x06: ReadByte
//!
//!    [0x08 0x0f 0x00 0x1e ]
//!      |    |    \----\----- Little-endian encoded value 0x1e00
//!      |    \--------------- Register 0x0f: TempSensorLimit
//!      \-------------------- Opcode 0x08: WriteWord
//!
//!    [0x0a 0x0b 0x0c 0x00 0x00 0x00 0x00 ...]
//!      |    |    |    \----\----\----\----\---- 12 bytes of data
//!      |    |    \----------------------------- Len 0x0c: 12 byte block
//!      |    \---------------------------------- Register 0x0b: LedCycleColors
//!      \--------------------------------------- Opcode 0x0a: WriteBlock
//!
//!    [0x0b 0x02 0x08]
//!      |    |    \---------- Len 0x08: 8 byte block to read
//!      |    \--------------- Register 0x02: ProductName
//!      \-------------------- Opcode 0x0b: ReadBlock
//!

use errors::*;

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

pub trait Register : Into<u8> + Copy {
    fn size(&self) -> usize;
}

pub trait Value<R: Register> : Sized + Clone {
    type DecodeError;

    fn decode(register: R, data: &[u8]) -> Result<Self>;

    fn encode(&self, buf: &mut [u8]) -> Option<usize>;
}

#[derive(Debug)]
pub enum Command<R,V> {
    Read(R),
    Write(R,V),
}

impl<R: Register, V: Value<R>> Command<R,V> {
    fn opcode(&self) -> Opcode {
        match self {
            &Command::Read(ref register) => match register.size() {
                1 => Opcode::ReadByte,
                2 => Opcode::ReadWord,
                _ => Opcode::ReadBlock,
            },
            &Command::Write(ref register, _) => match register.size() {
                1 => Opcode::WriteByte,
                2 => Opcode::WriteWord,
                _ => Opcode::WriteBlock,
            }
        }
    }

    fn register(&self) -> R {
        match self {
            &Command::Read(register) => register,
            &Command::Write(register, _) => register,
        }
    }

    fn encode(&self, buf: &mut [u8]) -> Option<usize> {
        buf[0] = self.opcode() as u8;
        buf[1] = self.register().into();

        match self {
            &Command::Read(register) => {
                match self.opcode() {
                    Opcode::ReadByte => Some(2),
                    Opcode::ReadWord => Some(2),
                    Opcode::ReadBlock => {
                        buf[2] = register.size() as u8;
                        Some(3)
                    },
                    _ => None
                }
            },
            &Command::Write(register, ref value) => {
                match self.opcode() {
                    Opcode::WriteByte => {
                        value.encode(&mut buf[2..3]);
                        Some(3)
                    },
                    Opcode::WriteWord => {
                        value.encode(&mut buf[2..4]);
                        Some(4)
                    },
                    Opcode::WriteBlock => {
                        buf[2] = register.size() as u8;
                        value.encode(&mut buf[3..3+register.size()]);
                        Some(2 + register.size())
                    },
                    _ => None
                }
            }
        }
    }

    fn len(&self) -> usize {
        match self.opcode() {
            Opcode::ReadByte => 2,
            Opcode::ReadWord => 2,
            Opcode::ReadBlock => 3,
            Opcode::WriteByte => 3,
            Opcode::WriteWord => 4,
            Opcode::WriteBlock => 3 + self.register().size(),
        }
    }
}

pub const PACKET_SIZE: usize = 64;
pub const FIRST_COMMAND_ID: u8 = 20;

#[derive(Debug)]
pub struct TxPacket<R,V> {
    first_command_id: u8,
    commands: Vec<Command<R,V>>,
}

impl<R: Register, V: Value<R>> TxPacket<R,V> {
    pub fn new(first_command_id: u8, commands: Vec<Command<R,V>>) -> TxPacket<R,V> {
        TxPacket { first_command_id, commands }
    }

    pub fn encode(self: &TxPacket<R,V>) -> Option<Vec<u8>> {
        let len = self.len();
        let mut buf: Vec<u8> = vec![0; len];
        buf[0] = len as u8 - 1;

        let mut i = 1;
        let mut command_id = self.first_command_id;

        for c in self.commands.iter() {
            buf[i] = command_id;
            i += 1;
            match buf.get_mut(i .. i + c.len()) {
                Some(slice) => c.encode(slice),
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
pub enum RxCommand<R, V> {
    Read(R, V),
    Write(R),
}

impl<R: Register, V: Value<R>> RxCommand<R, V> {
    fn decode_read(register: R, data: &[u8]) -> Result<RxCommand<R, V>> {
        let buf = match register.size() {
            1 => &data[0..1],
            2 => &data[0..2],
            len @ _ if len == data[0] as usize => &data[1..len+2],
            _ => return Err("Invalid length byte for block read".into()),
        };
        Ok(RxCommand::Read(register, V::decode(register, buf)?))
    }

    fn len(&self) -> usize {
        match self {
            &RxCommand::Read(register, _) => match register.size() {
                1 => 2,
                2 => 3,
                len @ _ => len + 2,
            },
            &RxCommand::Write(_) => 1,
        }
    }
}

#[derive(Debug)]
pub struct RxPacket<R,V>(Vec<RxCommand<R,V>>);

impl<R: Register, V: Value<R>> RxPacket<R,V> {
    pub fn decode(tx_packet: TxPacket<R,V>, data: &[u8]) -> Result<RxPacket<R,V>> {
        let mut rxpacket = RxPacket(Vec::new());

        let mut command_id = tx_packet.first_command_id;
        let mut i = 0;
        for c in tx_packet.commands.iter() {
            if data[i] != command_id {
                println!("Bad command ID {}", data[i]);
                return Err("Bad command ID".into());
            }

            let rxcommand = match c {
                &Command::Read(register) => RxCommand::decode_read(register, &data[i + 2 .. ])?,
                &Command::Write(register, _) => RxCommand::Write(register),
            };

            command_id += 1;
            i += rxcommand.len() + 1;
            rxpacket.0.push(rxcommand);
        }

        Ok(rxpacket)
    }

    pub fn read_values(&self) -> Vec<V> {
        self.0.iter().filter_map(|rxcommand| {
            match rxcommand {
                &RxCommand::Read(_, ref value) => Some(value.clone()),
                _ => None,
            }
        }).collect()
    }
}
