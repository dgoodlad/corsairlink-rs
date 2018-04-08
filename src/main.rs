extern crate hidapi;

mod phy;
mod h110i;

//use phy::Encodable;

// H110i
const VENDOR_ID: u16 = 0x1b1c;
const PRODUCT_ID: u16 = 0x0c04;

fn main() {
    let api = hidapi::HidApi::new().expect("Failed to initialize hidapi");
    let hiddev = api.open(VENDOR_ID, PRODUCT_ID).expect("Failed to open hid device");
    //let dev = phy::CorsairDevice::new(hiddev);

    let commands = vec![
        h110i::Command::Read(h110i::Register::DeviceId),
        h110i::Command::Read(h110i::Register::FirmwareVersion),
        h110i::Command::Read(h110i::Register::ProductName),
    ];

    let packet = h110i::TxPacket::new(20, commands);

    //println!("Len: {:?}", packet.len());
    //println!("Encoded: {:?}", packet.encode());

    match hiddev.write(packet.encode().unwrap().as_slice()) {
        Ok(len) => println!("Wrote {} bytes", len),
        Err(e) => panic!(e),
    }

    let mut buf = vec![0u8; 64];
    match hiddev.read_timeout(&mut buf[..], 1000) {
        Ok(len) => println!("Read {} bytes", len),
        Err(e) => panic!(e),
    }

    //let data = vec![20, 0x07, 0x42,
    //                21, 0x09, 0x01, 0x02,
    //                22, 0x0a, 0x08, 0x48, 0x31, 0x31, 0x30, 0x69, 0x00, 0x00, 0x00 // H110i
    //];

    println!("Got data: {:?}", buf);
    let decoded = h110i::RxPacket::decode(packet, &buf[..]);
    println!("Decoded: {:?}", decoded);

    let packet2 = h110i::TxPacket::new(23, vec![
        h110i::Command::Read(h110i::Register::TempSensorCount),
        h110i::Command::Write(h110i::Register::TempSensorSelect, h110i::RegisterValue::TempSensorSelect(0)),
        h110i::Command::Read(h110i::Register::TempSensorValue),
    ]);

    match hiddev.write(packet2.encode().unwrap().as_slice()) {
        Ok(len) => println!("Wrote {} bytes", len),
        Err(e) => panic!(e),
    }

    let mut buf2 = vec![0u8; 64];
    match hiddev.read_timeout(&mut buf2[..], 1000) {
        Ok(len) => println!("Read {} bytes", len),
        Err(e) => panic!(e),
    }

    println!("Decoded: {:?}", h110i::RxPacket::decode(packet2, &buf2[..]));
}
