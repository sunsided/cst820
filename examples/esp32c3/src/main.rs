//! Reads touches from a CST820 and prints them over the serial console.
//!
//! Wiring: SDA = GPIO4, SCL = GPIO5, RST = GPIO6, INT = GPIO7.

#![no_std]
#![no_main]

use cst820::Cst820;
use cst820::registers::IrqCtl;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::main;
use esp_hal::time::Rate;
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut delay = Delay::new();

    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .expect("valid I2C config")
    .with_sda(peripherals.GPIO4)
    .with_scl(peripherals.GPIO5);
    let mut rst = Output::new(peripherals.GPIO6, Level::High, OutputConfig::default());
    let int = Input::new(
        peripherals.GPIO7,
        InputConfig::default().with_pull(Pull::Up),
    );

    cst820::reset(&mut rst, &mut delay).unwrap();
    let mut touch = Cst820::new(i2c);
    let chip_id = touch.chip_id().expect("chip responds after reset");
    if chip_id != cst820::CST820_CHIP_ID {
        panic!("unexpected chip ID 0x{chip_id:02X}");
    }
    touch
        .configure_interrupts(IrqCtl::new().with_en_change(true).with_en_motion(true))
        .unwrap();
    println!("CST820 ok chip_id=0x{chip_id:02X}");

    loop {
        while int.is_high() {}
        match touch.read_touch() {
            Ok(t) => println!(
                "touch x={} y={} fingers={} ev={:?} gesture={:?}",
                t.x, t.y, t.fingers, t.event, t.gesture
            ),
            Err(e) => println!("read failed: {e:?}"),
        }
        while int.is_low() {}
    }
}
