extern crate hidapi;
mod phy;
mod h110i;

// H110i
const VENDOR_ID: u16 = 0x1b1c;
const PRODUCT_ID: u16 = 0x0c04;

fn main() {
    let api = hidapi::HidApi::new().expect("Failed to initialize hidapi");
    let hiddev = api.open(VENDOR_ID, PRODUCT_ID).expect("Failed to open hid device");
    let dev = phy::CorsairDevice::new(hiddev);

    let commands = vec![
        h110i::Command::Read(h110i::Register::DeviceId),
        h110i::Command::Read(h110i::Register::FirmwareVersion),
        h110i::Command::Read(h110i::Register::ProductName),
    ];

    match dev.encode_commands(&commands) {
        
    }
}
