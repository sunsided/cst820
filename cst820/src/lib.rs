//! # Hynitron CST820 touch controller driver
//!
//! > A platform-agnostic driver to interface with the Hynitron CST820 touch controller.
//!
//! This driver was built using [`embedded-hal`] 1.0 and [`cst820-registers`]. Enable the
//! `async` feature for an equivalent driver on [`embedded-hal-async`] in the `asynch` module.
//!
//! ## Standby
//!
//! The chip enters standby after [`registers::AutoSleepTime`] seconds
//! (default: 2) without touch, and then does not acknowledge on the bus until touched.
//! Read the chip after its interrupt fires, or disable auto-sleep with
//! [`Cst820::set_auto_sleep`] right after reset.
//!
//! ## Limitations
//!
//! The CST820 supports two-point touch, but the register layout of the second point is
//! undocumented. Only the first point is reported.
//!
//! [`embedded-hal`]: https://crates.io/crates/embedded-hal
//! [`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async
//! [`cst820-registers`]: https://crates.io/crates/cst820-registers

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![forbid(unsafe_code)]
// Enables the `doc_cfg` feature when the `docsrs` configuration attribute is defined.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod asynch;
mod touch;

pub use cst820_registers as registers;
pub use cst820_registers::{CST820_CHIP_ID, DEFAULT_DEVICE_ADDRESS, Gesture, TouchEvent};
pub use touch::Touch;

use cst820_registers::prelude::*;
use cst820_registers::{
    AutoSleepTime, ChipId, DisAutoSleep, FwVersion, IrqCtl, MotionMask, ProjId, SleepMode,
};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::I2c;

/// How long `RST` is held low during [`reset`], in microseconds.
///
/// The datasheet requires at least 0.1 ms; 1 ms adds margin.
pub const RESET_PULSE_US: u32 = 1_000;

/// How long to wait after releasing `RST` before talking to the chip, in milliseconds.
///
/// The datasheet requires at least 100 ms (`Tron`).
pub const BOOT_DELAY_MS: u32 = 100;

/// Resets the chip via its active-low `RST` pin and waits until it has booted.
pub fn reset<P, D>(rst: &mut P, delay: &mut D) -> Result<(), P::Error>
where
    P: OutputPin,
    D: DelayNs,
{
    rst.set_low()?;
    delay.delay_us(RESET_PULSE_US);
    rst.set_high()?;
    delay.delay_ms(BOOT_DELAY_MS);
    Ok(())
}

/// CST820 driver.
#[derive(Debug)]
pub struct Cst820<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> Cst820<I2C>
where
    I2C: I2c,
{
    /// Creates a driver for a chip at [`DEFAULT_DEVICE_ADDRESS`]. Performs no bus access.
    pub fn new(i2c: I2C) -> Self {
        Self::with_address(i2c, DEFAULT_DEVICE_ADDRESS)
    }

    /// Creates a driver for a chip at the specified 7-bit address. Performs no bus access.
    pub fn with_address(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Consumes the driver and returns the I²C bus.
    pub fn release(self) -> I2C {
        self.i2c
    }

    /// Reads the chip ID.
    pub fn chip_id(&mut self) -> Result<u8, I2C::Error> {
        Ok(self.read_register::<ChipId>()?.value())
    }

    /// Checks whether the chip ID matches [`CST820_CHIP_ID`].
    pub fn identify(&mut self) -> Result<bool, I2C::Error> {
        Ok(self.chip_id()? == CST820_CHIP_ID)
    }

    /// Reads the project ID.
    pub fn project_id(&mut self) -> Result<u8, I2C::Error> {
        Ok(self.read_register::<ProjId>()?.value())
    }

    /// Reads the firmware version.
    pub fn firmware_version(&mut self) -> Result<u8, I2C::Error> {
        Ok(self.read_register::<FwVersion>()?.value())
    }

    /// Enables gesture actions: continuous scrolling and double click.
    ///
    /// Double clicks are only reported when enabled here.
    pub fn configure_gestures(&mut self, mask: MotionMask) -> Result<(), I2C::Error> {
        self.write_register(mask)
    }

    /// Reads the current touch state in a single burst.
    pub fn read_touch(&mut self) -> Result<Touch, I2C::Error> {
        let mut buffer = [0; Touch::LEN];
        self.i2c
            .write_read(self.address, &[Touch::START.addr()], &mut buffer)?;
        Ok(Touch::from_bytes(&buffer))
    }

    /// Selects which events pulse the interrupt pin.
    pub fn configure_interrupts(&mut self, irq: IrqCtl) -> Result<(), I2C::Error> {
        self.write_register(irq)
    }

    /// Enables automatic standby after the specified number of seconds without touch,
    /// or disables it when `None`.
    pub fn set_auto_sleep(&mut self, seconds: Option<u8>) -> Result<(), I2C::Error> {
        match seconds {
            Some(seconds) => {
                self.write_register(AutoSleepTime::new().with_value(seconds))?;
                self.write_register(DisAutoSleep::new())
            }
            None => self.write_register(DisAutoSleep::new().with_value(1)),
        }
    }

    /// Puts the chip into deep sleep. Only a [`reset`] wakes it up again.
    pub fn sleep(&mut self) -> Result<(), I2C::Error> {
        self.write_register(SleepMode::deep_sleep())
    }

    /// Reads a single register.
    pub fn read_register<R>(&mut self) -> Result<R, I2C::Error>
    where
        R: ReadableRegister,
    {
        let mut buffer = [0];
        self.i2c
            .write_read(self.address, &[*R::REGISTER_ADDRESS], &mut buffer)?;
        Ok(R::from_bits(buffer[0]))
    }

    /// Writes a single register.
    pub fn write_register<R>(&mut self, register: R) -> Result<(), I2C::Error>
    where
        R: WritableRegister,
    {
        self.i2c
            .write(self.address, &[*R::REGISTER_ADDRESS, register.to_bits()])
    }

    /// Reads, modifies and writes back a single register.
    pub fn modify_register<R, F>(&mut self, f: F) -> Result<(), I2C::Error>
    where
        R: ReadableRegister + WritableRegister,
        F: FnOnce(R) -> R,
    {
        let register = self.read_register::<R>()?;
        self.write_register(f(register))
    }
}
