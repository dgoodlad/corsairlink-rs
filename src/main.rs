#[macro_use]
extern crate error_chain;

extern crate corsairlink;
extern crate libusb;

use corsairlink::devices::cooler::h110i;
use corsairlink::errors::*;

quick_main!(run);

fn run() -> Result<()> {
    let context = libusb::Context::new().unwrap();

    let mut cooler = h110i::Device::open(&context)?;
    cooler.get_metadata()?;
    println!("Cooler: {:?}", cooler);

    cooler.poll_leds()?;
    cooler.poll_temperatures()?;
    cooler.poll_fans()?;

    println!("Temperature: {}", cooler.temperatures[0]);
    println!("Fan Speeds: {}, {}, {}", cooler.fan_speeds[0], cooler.fan_speeds[1], cooler.fan_speeds[2]);
    println!("Fan Modes: {:?}", cooler.fan_modes);
    println!("LED Modes: {:?}", cooler.led_modes[0]);
    println!("LED Colors: {:?}", cooler.led_colors[0]);
    println!("LED Cycle Colors: {:?}", cooler.led_cycle_colors[0]);

    println!("");
    println!("Setting color cycle to magenta, green, blue, white");
    cooler.set_led_colors(0, [
        h110i::RgbColor(255, 0, 255),
        h110i::RgbColor(0, 255, 0),
        h110i::RgbColor(0, 0, 255),
        h110i::RgbColor(255, 255, 255),
    ])?;
    cooler.set_led_mode(h110i::LedMode::four_color_cycle_mode(7))?;
    cooler.poll_leds()?;
    println!("LED Colors: {:?}", cooler.led_colors[0]);
    println!("LED Cycle Colors: {:?}", cooler.led_cycle_colors[0]);

    Ok(())
}
