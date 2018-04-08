extern crate hidapi;

const READ_TIMEOUT: i32 = 1000;
const PACKET_SIZE: usize = 64;

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

    pub fn write_packet<T: Encodable>(&self, packet: &T) -> CorsairResult<usize> {
        let data = packet.encode();
        self.write(&data[..])
    }
}
