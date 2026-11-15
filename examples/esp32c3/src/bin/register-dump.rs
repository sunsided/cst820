//! Dumps CST820 registers to answer open questions on real hardware.
//!
//! - At boot, prints the identification and configuration registers untouched, which
//!   shows the power-on defaults (e.g. `IrqCtl` at `0xFA`).
//! - Then polls `0x00..=0x0F` every 20 ms and prints the block whenever it changes.
//!   Touch with one and two fingers to see the event bits and any second touch point.
//! - Read failures are printed once per transition, which shows whether the chip stops
//!   answering in standby (about 2 s after the last touch).
//!
//! Wiring: SDA = GPIO4, SCL = GPIO5, RST = GPIO6. INT is not used.

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::main;
use esp_hal::time::{Instant, Rate};
use esp_println::{print, println};

esp_bootloader_esp_idf::esp_app_desc!();

/// Register blocks printed once at boot: (first register, length).
const CONFIG_BLOCKS: [(u8, usize); 3] = [(0xA7, 3), (0xB0, 4), (0xEC, 19)];

/// The polled block: gesture, finger count, both potential touch points and padding.
const POLL_START: u8 = 0x00;
const POLL_LEN: usize = 16;
const POLL_INTERVAL_MS: u32 = 20;

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    let mut i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(100)),
    )
    .expect("valid I2C config")
    .with_sda(peripherals.GPIO4)
    .with_scl(peripherals.GPIO5);
    let mut rst = Output::new(peripherals.GPIO6, Level::High, OutputConfig::default());

    cst820::reset(&mut rst, &mut Delay::new()).unwrap();
    println!(
        "register dump: address 0x{:02X}",
        cst820::DEFAULT_DEVICE_ADDRESS
    );

    for (start, len) in CONFIG_BLOCKS {
        let mut buffer = [0u8; 19];
        let buffer = &mut buffer[..len];

        match i2c.write_read(cst820::DEFAULT_DEVICE_ADDRESS, &[start], buffer) {
            Ok(()) => print_block("config", start, buffer),
            Err(e) => println!("config {start:02X}: read failed: {e:?}"),
        }
    }

    let mut last = [0u8; POLL_LEN];
    let mut have_last = false;
    let mut failing = false;

    loop {
        let mut block = [0u8; POLL_LEN];

        match i2c.write_read(cst820::DEFAULT_DEVICE_ADDRESS, &[POLL_START], &mut block) {
            Ok(()) => {
                if failing {
                    println!("t={}ms bus ok again", now_ms());
                    failing = false;
                }

                if !have_last || block != last {
                    print!("t={}ms ", now_ms());
                    print_block("touch", POLL_START, &block);
                    last = block;
                    have_last = true;
                }
            }

            Err(e) => {
                if !failing {
                    println!("t={}ms read failed: {e:?}", now_ms());
                    failing = true;
                }
            }
        }

        delay.delay_millis(POLL_INTERVAL_MS);
    }
}

fn now_ms() -> u64 {
    Instant::now().duration_since_epoch().as_millis()
}

fn print_block(label: &str, start: u8, bytes: &[u8]) {
    print!(
        "{label} {start:02X}-{:02X}:",
        usize::from(start) + bytes.len() - 1
    );

    for byte in bytes {
        print!(" {byte:02X}");
    }

    println!();
}
