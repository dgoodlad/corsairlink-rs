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
            &Command::Read(_) => Some(2),
            &Command::Write(register, ref value) => {
                value.encode(buf[2..].as_mut());
                Some(2 + register.size())
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            &Command::Read(_) => 2,
            &Command::Write(register, _) => 2 + register.size(),
        }
    }
}

pub const PACKET_SIZE: usize = 64;
const FIRST_COMMAND_ID: u8 = 20;

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
        buf[0] = len as u8;

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
        Ok(RxCommand::Read(
            register,
            V::decode(register, data)?
        ))
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
