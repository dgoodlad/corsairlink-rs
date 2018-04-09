#[macro_use]
extern crate error_chain;

extern crate corsair;
extern crate libusb;

use corsair::devices::cooler::h110i;
use corsair::errors::*;

quick_main!(run);

fn run() -> Result<()> {
    let context = libusb::Context::new().unwrap();

    let mut cooler = h110i::Device::open(&context)?;
    cooler.get_metadata()?;
    println!("Cooler: {:?}", cooler);

    cooler.poll_temperatures()?;
    println!("Temperature: {}", cooler.temperatures[0]);

    cooler.poll_fans()?;
    println!("Cooler: {:?}", cooler);

    println!("{}, {}, {}", cooler.fan_speeds[0], cooler.fan_speeds[1], cooler.fan_speeds[2]);

    Ok(())
}
