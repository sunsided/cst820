//! Async driver on `embedded-hal-async`.
//!
//! Mirrors the blocking [`Cst820`](crate::Cst820) and [`reset`](crate::reset).

use crate::{BOOT_DELAY_MS, CST820_CHIP_ID, DEFAULT_DEVICE_ADDRESS, RESET_PULSE_US, Touch};
use cst820_registers::prelude::*;
use cst820_registers::{
    AutoSleepTime, ChipId, DisAutoSleep, FwVersion, IrqCtl, MotionMask, ProjId, SleepMode,
};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::I2c;

/// Resets the chip via its active-low `RST` pin and waits until it has booted.
pub async fn reset<P, D>(rst: &mut P, delay: &mut D) -> Result<(), P::Error>
where
    P: OutputPin,
    D: DelayNs,
{
    rst.set_low()?;
    delay.delay_us(RESET_PULSE_US).await;
    rst.set_high()?;
    delay.delay_ms(BOOT_DELAY_MS).await;
    Ok(())
}

/// Async CST820 driver.
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
    pub async fn chip_id(&mut self) -> Result<u8, I2C::Error> {
        Ok(self.read_register::<ChipId>().await?.value())
    }

    /// Checks whether the chip ID matches [`CST820_CHIP_ID`].
    pub async fn identify(&mut self) -> Result<bool, I2C::Error> {
        Ok(self.chip_id().await? == CST820_CHIP_ID)
    }

    /// Reads the project ID.
    pub async fn project_id(&mut self) -> Result<u8, I2C::Error> {
        Ok(self.read_register::<ProjId>().await?.value())
    }

    /// Reads the firmware version.
    pub async fn firmware_version(&mut self) -> Result<u8, I2C::Error> {
        Ok(self.read_register::<FwVersion>().await?.value())
    }

    /// Enables gesture actions: continuous scrolling and double click.
    ///
    /// Double clicks are only reported when enabled here.
    pub async fn configure_gestures(&mut self, mask: MotionMask) -> Result<(), I2C::Error> {
        self.write_register(mask).await
    }

    /// Reads the current touch state in a single burst.
    pub async fn read_touch(&mut self) -> Result<Touch, I2C::Error> {
        let mut buffer = [0; Touch::LEN];
        self.i2c
            .write_read(self.address, &[Touch::START.addr()], &mut buffer)
            .await?;
        Ok(Touch::from_bytes(&buffer))
    }

    /// Selects which events pulse the interrupt pin.
    pub async fn configure_interrupts(&mut self, irq: IrqCtl) -> Result<(), I2C::Error> {
        self.write_register(irq).await
    }

    /// Enables automatic standby after the specified number of seconds without touch,
    /// or disables it when `None`.
    pub async fn set_auto_sleep(&mut self, seconds: Option<u8>) -> Result<(), I2C::Error> {
        match seconds {
            Some(seconds) => {
                self.write_register(AutoSleepTime::new().with_value(seconds))
                    .await?;
                self.write_register(DisAutoSleep::new()).await
            }
            None => self.write_register(DisAutoSleep::new().with_value(1)).await,
        }
    }

    /// Puts the chip into deep sleep. Only a [`reset`] wakes it up again.
    pub async fn sleep(&mut self) -> Result<(), I2C::Error> {
        self.write_register(SleepMode::deep_sleep()).await
    }

    /// Reads a single register.
    pub async fn read_register<R>(&mut self) -> Result<R, I2C::Error>
    where
        R: ReadableRegister,
    {
        let mut buffer = [0];
        self.i2c
            .write_read(self.address, &[*R::REGISTER_ADDRESS], &mut buffer)
            .await?;
        Ok(R::from_bits(buffer[0]))
    }

    /// Writes a single register.
    pub async fn write_register<R>(&mut self, register: R) -> Result<(), I2C::Error>
    where
        R: WritableRegister,
    {
        self.i2c
            .write(self.address, &[*R::REGISTER_ADDRESS, register.to_bits()])
            .await
    }

    /// Reads, modifies and writes back a single register.
    pub async fn modify_register<R, F>(&mut self, f: F) -> Result<(), I2C::Error>
    where
        R: ReadableRegister + WritableRegister,
        F: FnOnce(R) -> R,
    {
        let register = self.read_register::<R>().await?;
        self.write_register(f(register)).await
    }
}
