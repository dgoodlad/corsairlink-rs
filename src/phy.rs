extern crate hidapi;

const READ_TIMEOUT: i32 = 1000;
const PACKET_SIZE: usize = 64;
const FIRST_COMMAND_ID: u8 = 20;

type CorsairResult<T> = hidapi::HidResult<T>;

pub trait Encodable : Sized {
    fn encode_into(&self, buf: &mut [u8]) -> Option<usize>;
    fn len(&self) -> usize;

    fn encode(&self) -> Vec<u8> {
        let mut v: Vec<u8> = vec![0; self.len()];
        self.encode_into(v.as_mut_slice());
        v
    }
}

pub struct CorsairDevice<'a> {
    dev: hidapi::HidDevice<'a>,
    last_command_id: u8,
}

impl<'a> CorsairDevice<'a> {
    pub fn new(dev: hidapi::HidDevice<'a>) -> CorsairDevice<'a> {
        CorsairDevice { dev, last_command_id: 20 }
    }

    pub fn write(&self, data: &[u8]) -> CorsairResult<usize> {
        self.dev.write(data)
    }

    pub fn read(&self, buf: &mut [u8; PACKET_SIZE]) -> CorsairResult<usize> {
        self.dev.read_timeout(buf, READ_TIMEOUT)
    }

    pub fn encode_commands<T: Encodable>(&self, commands: &[T]) -> Option<Vec<u8>> {
        let mut buf: Vec<u8> = vec!(0; PACKET_SIZE);
        let mut i = 1;
        let command_id = FIRST_COMMAND_ID;

        for c in commands {
            buf[i] = command_id + i as u8;
            i += 1;
            match buf.get_mut(i .. i + c.len() + 1) {
                Some(slice) => c.encode_into(slice),
                None => return None
            };
            i += c.len();
        }

        buf[0] = i as u8;

        Some(buf)
    }

    pub fn write_commands<T: Encodable>(&self, commands: &[T]) -> CorsairResult<usize> {
        match self.encode_commands(commands) {
            Some(data) => self.write(&data[..]),
            None => Err("Failed to encode commands")
        }
    }
}
