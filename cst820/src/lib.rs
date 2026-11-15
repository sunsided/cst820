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
#![no_std]
#![forbid(unsafe_code)]
// Enables the `doc_cfg` feature when the `docsrs` configuration attribute is defined.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
mod driver;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod asynch;
mod touch;

pub use cst820_registers as registers;
pub use cst820_registers::{CST820_CHIP_ID, DEFAULT_DEVICE_ADDRESS, Gesture, TouchEvent};
pub use touch::Touch;

/// How long `RST` is held low during [`reset`], in microseconds.
///
/// The datasheet requires at least 0.1 ms; 1 ms adds margin.
pub const RESET_PULSE_US: u32 = 1_000;

/// How long to wait after releasing `RST` before talking to the chip, in milliseconds.
///
/// The datasheet requires at least 100 ms (`Tron`).
pub const BOOT_DELAY_MS: u32 = 100;

driver!(
    doc = "CST820 driver.",
    i2c = embedded_hal::i2c::I2c,
    delay = embedded_hal::delay::DelayNs,
    async = [],
    await = [],
);
