//! The driver, written once for both the blocking and the async API.
//!
//! [`driver!`] expands to [`reset`](crate::reset) and [`Cst820`](crate::Cst820) in the
//! crate root, and to their async counterparts in `asynch`. The async expansion only
//! differs in the `async` keyword on functions and `.await` after bus and delay calls.

/// Generates `reset`, the `Cst820` driver struct and its methods.
///
/// - `i2c` / `delay`: the I²C bus and delay traits to build on.
/// - `async`: empty, or `async` to make the methods async.
/// - `await`: empty, or `.await` to await bus and delay calls.
macro_rules! driver {
    (
        doc = $doc:literal,
        i2c = $i2c:path,
        delay = $delay:path,
        async = [$($asy:tt)*],
        await = [$($aw:tt)*] $(,)?
    ) => {
        /// Resets the chip via its active-low `RST` pin and waits until it has booted.
        pub $($asy)* fn reset<P, D>(rst: &mut P, delay: &mut D) -> Result<(), P::Error>
        where
            P: embedded_hal::digital::OutputPin,
            D: $delay,
        {
            rst.set_low()?;
            delay.delay_us($crate::RESET_PULSE_US) $($aw)*;
            rst.set_high()?;
            delay.delay_ms($crate::BOOT_DELAY_MS) $($aw)*;
            Ok(())
        }

        #[doc = $doc]
        #[derive(Debug)]
        pub struct Cst820<I2C> {
            i2c: I2C,
            address: u8,
        }

        impl<I2C> Cst820<I2C>
        where
            I2C: $i2c,
        {
            /// Creates a driver for a chip at
            /// [`DEFAULT_DEVICE_ADDRESS`](crate::DEFAULT_DEVICE_ADDRESS). Performs no bus
            /// access.
            pub fn new(i2c: I2C) -> Self {
                Self::with_address(i2c, $crate::DEFAULT_DEVICE_ADDRESS)
            }

            /// Creates a driver for a chip at the specified 7-bit address. Performs no bus
            /// access.
            pub fn with_address(i2c: I2C, address: u8) -> Self {
                Self { i2c, address }
            }

            /// Consumes the driver and returns the I²C bus.
            pub fn release(self) -> I2C {
                self.i2c
            }

            /// Reads the chip ID.
            pub $($asy)* fn chip_id(&mut self) -> Result<u8, I2C::Error> {
                Ok(self.read_register::<$crate::registers::ChipId>() $($aw)* ?.value())
            }

            /// Checks whether the chip ID matches [`CST820_CHIP_ID`](crate::CST820_CHIP_ID).
            pub $($asy)* fn identify(&mut self) -> Result<bool, I2C::Error> {
                Ok(self.chip_id() $($aw)* ? == $crate::CST820_CHIP_ID)
            }

            /// Reads the project ID.
            pub $($asy)* fn project_id(&mut self) -> Result<u8, I2C::Error> {
                Ok(self.read_register::<$crate::registers::ProjId>() $($aw)* ?.value())
            }

            /// Reads the firmware version.
            pub $($asy)* fn firmware_version(&mut self) -> Result<u8, I2C::Error> {
                Ok(self.read_register::<$crate::registers::FwVersion>() $($aw)* ?.value())
            }

            /// Enables gesture actions: continuous scrolling and double click.
            ///
            /// Double clicks are only reported when enabled here.
            pub $($asy)* fn configure_gestures(
                &mut self,
                mask: $crate::registers::MotionMask,
            ) -> Result<(), I2C::Error> {
                self.write_register(mask) $($aw)*
            }

            /// Reads the current touch state in a single burst.
            pub $($asy)* fn read_touch(&mut self) -> Result<$crate::Touch, I2C::Error> {
                let mut buffer = [0; $crate::Touch::LEN];
                self.i2c
                    .write_read(self.address, &[$crate::Touch::START.addr()], &mut buffer)
                    $($aw)* ?;
                Ok($crate::Touch::from_bytes(&buffer))
            }

            /// Selects which events pulse the interrupt pin.
            pub $($asy)* fn configure_interrupts(
                &mut self,
                irq: $crate::registers::IrqCtl,
            ) -> Result<(), I2C::Error> {
                self.write_register(irq) $($aw)*
            }

            /// Enables automatic standby after the specified number of seconds without
            /// touch, or disables it when `None`.
            pub $($asy)* fn set_auto_sleep(&mut self, seconds: Option<u8>) -> Result<(), I2C::Error> {
                use $crate::registers::{AutoSleepTime, DisAutoSleep};

                match seconds {
                    Some(seconds) => {
                        self.write_register(AutoSleepTime::new().with_value(seconds)) $($aw)* ?;
                        self.write_register(DisAutoSleep::new()) $($aw)*
                    }
                    None => self.write_register(DisAutoSleep::new().with_value(1)) $($aw)*,
                }
            }

            /// Puts the chip into deep sleep. Only a [`reset`] wakes it up again.
            pub $($asy)* fn sleep(&mut self) -> Result<(), I2C::Error> {
                self.write_register($crate::registers::SleepMode::deep_sleep()) $($aw)*
            }

            /// Reads a single register.
            pub $($asy)* fn read_register<R>(&mut self) -> Result<R, I2C::Error>
            where
                R: $crate::registers::ReadableRegister,
            {
                let mut buffer = [0];
                self.i2c
                    .write_read(self.address, &[*R::REGISTER_ADDRESS], &mut buffer)
                    $($aw)* ?;
                Ok(R::from_bits(buffer[0]))
            }

            /// Writes a single register.
            pub $($asy)* fn write_register<R>(&mut self, register: R) -> Result<(), I2C::Error>
            where
                R: $crate::registers::WritableRegister,
            {
                self.i2c
                    .write(self.address, &[*R::REGISTER_ADDRESS, register.to_bits()])
                    $($aw)*
            }

            /// Reads, modifies and writes back a single register.
            pub $($asy)* fn modify_register<R, F>(&mut self, f: F) -> Result<(), I2C::Error>
            where
                R: $crate::registers::ReadableRegister + $crate::registers::WritableRegister,
                F: FnOnce(R) -> R,
            {
                let register = self.read_register::<R>() $($aw)* ?;
                self.write_register(f(register)) $($aw)*
            }
        }
    };
}
