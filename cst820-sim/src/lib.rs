//! # Hynitron CST820 behavioral model
//!
//! A register-level model of the CST820 touch controller, used in two places:
//!
//! - as a fake I²C bus ([`FakeI2c`]) in host tests of the `cst820` driver, and
//! - wrapped into a [Wokwi](https://wokwi.com) custom chip (`sim/wokwi-chip`).
//!
//! The model implements the documented I²C protocol (register pointer write followed by
//! auto-incrementing reads or writes), the touch registers, interrupt generation,
//! automatic standby and the sleep command.
//!
//! ## Assumptions
//!
//! Behavior that is not documented in the datasheet follows community reports for the
//! CST816 family: the chip does not acknowledge its address while in standby or deep
//! sleep. A touch wakes it from standby; only a reset wakes it from deep sleep.

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod fake_i2c;

pub use fake_i2c::FakeI2c;

use cst820_registers::{
    CST820_CHIP_ID, DEFAULT_DEVICE_ADDRESS, Gesture, IrqCtl, MotionMask, RegisterAddress as Reg,
    SleepMode, TouchEvent,
};

/// The power mode of the model.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PowerMode {
    /// Scanning quickly; responds on the bus.
    Dynamic,
    /// Entered after [`AUTO_SLEEP_TIME`](Reg::AUTO_SLEEP_TIME) seconds without touch.
    /// Does not respond on the bus; a touch wakes it.
    Standby,
    /// Entered by writing [`SleepMode::DEEP_SLEEP`]. Does not respond on the bus; only a
    /// reset wakes it.
    DeepSleep,
}

/// The firmware version reported by the model.
pub const MODEL_FW_VERSION: u8 = 0x01;

/// The default [`IrqCtl`] value of the model: interrupt on touch and on change.
///
/// The real power-on default is undocumented.
pub const MODEL_DEFAULT_IRQ_CTL: u8 = 0x60;

/// Register addresses that accept writes.
const WRITABLE: &[Reg] = &[
    Reg::SLEEP_MODE,
    Reg::MOTION_MASK,
    Reg::IRQ_PULSE_WIDTH,
    Reg::NOR_SCAN_PER,
    Reg::MOTION_SL_ANGLE,
    Reg::LP_AUTO_WAKE_TIME,
    Reg::LP_SCAN_TH,
    Reg::LP_SCAN_WIN,
    Reg::LP_SCAN_FREQ,
    Reg::LP_SCAN_IDAC,
    Reg::AUTO_SLEEP_TIME,
    Reg::IRQ_CTL,
    Reg::AUTO_RESET,
    Reg::LONG_PRESS_TIME,
    Reg::IO_CTL,
    Reg::DIS_AUTO_SLEEP,
];

/// A behavioral model of the CST820.
#[derive(Debug, Clone)]
pub struct Cst820Model {
    address: u8,
    regs: [u8; 256],
    pointer: u8,
    /// `true` until the first byte of a write transaction set the register pointer.
    expect_pointer: bool,
    touching: bool,
    mode: PowerMode,
    idle_ms: u32,
}

impl Default for Cst820Model {
    fn default() -> Self {
        Self::new()
    }
}

impl Cst820Model {
    /// Creates a model listening on [`DEFAULT_DEVICE_ADDRESS`], freshly out of reset.
    pub fn new() -> Self {
        Self::with_address(DEFAULT_DEVICE_ADDRESS)
    }

    /// Creates a model listening on the specified 7-bit address, freshly out of reset.
    pub fn with_address(address: u8) -> Self {
        let mut model = Self {
            address,
            regs: [0; 256],
            pointer: 0,
            expect_pointer: false,
            touching: false,
            mode: PowerMode::Dynamic,
            idle_ms: 0,
        };

        model.reset();
        model
    }

    /// Resets the chip, as by a low pulse on the `RST` pin.
    pub fn reset(&mut self) {
        use cst820_registers::{
            AutoSleepTime, IrqPulseWidth, LongPressTime, LpAutoWakeTime, LpScanFreq, LpScanTh,
            LpScanWin, NorScanPer,
        };

        self.regs = [0; 256];
        self.set(Reg::CHIP_ID, CST820_CHIP_ID);
        self.set(Reg::FW_VERSION, MODEL_FW_VERSION);
        self.set(Reg::XPOS_H, TouchEvent::None.into_bits() << 6);
        self.set(Reg::IRQ_PULSE_WIDTH, IrqPulseWidth::new().value());
        self.set(Reg::NOR_SCAN_PER, NorScanPer::new().value());
        self.set(Reg::LP_AUTO_WAKE_TIME, LpAutoWakeTime::new().value());
        self.set(Reg::LP_SCAN_TH, LpScanTh::new().value());
        self.set(Reg::LP_SCAN_WIN, LpScanWin::new().value());
        self.set(Reg::LP_SCAN_FREQ, LpScanFreq::new().value());
        self.set(Reg::AUTO_SLEEP_TIME, AutoSleepTime::new().value());
        self.set(Reg::LONG_PRESS_TIME, LongPressTime::new().value());
        self.set(Reg::IRQ_CTL, MODEL_DEFAULT_IRQ_CTL);
        self.pointer = 0;
        self.expect_pointer = false;
        self.touching = false;
        self.mode = PowerMode::Dynamic;
        self.idle_ms = 0;
    }

    /// The 7-bit I²C address of the model.
    pub fn address(&self) -> u8 {
        self.address
    }

    /// The current power mode.
    pub fn power_mode(&self) -> PowerMode {
        self.mode
    }

    /// Reads a register directly, bypassing the bus.
    pub fn register(&self, reg: Reg) -> u8 {
        self.regs[usize::from(reg.addr())]
    }

    /// The configured interrupt pulse width in microseconds.
    pub fn irq_pulse_width_us(&self) -> u32 {
        u32::from(self.register(Reg::IRQ_PULSE_WIDTH)) * 100
    }

    fn set(&mut self, reg: Reg, value: u8) {
        self.regs[usize::from(reg.addr())] = value;
    }

    fn irq_ctl(&self) -> IrqCtl {
        IrqCtl::from(self.register(Reg::IRQ_CTL))
    }

    /// Starts a transaction addressed to the model. Returns `true` to acknowledge.
    ///
    /// `is_read` selects a read (`true`) or write (`false`) transfer.
    pub fn i2c_start(&mut self, is_read: bool) -> bool {
        self.expect_pointer = !is_read;
        self.mode == PowerMode::Dynamic
    }

    /// Writes a byte within a write transfer. Returns `true` to acknowledge.
    ///
    /// The first byte sets the register pointer; subsequent bytes are written to
    /// consecutive registers. Writes to read-only registers are ignored.
    pub fn i2c_write(&mut self, byte: u8) -> bool {
        if self.mode != PowerMode::Dynamic {
            return false;
        }

        if self.expect_pointer {
            self.pointer = byte;
            self.expect_pointer = false;

            return true;
        }

        let addr = self.pointer;
        self.pointer = self.pointer.wrapping_add(1);

        if let Some(&reg) = WRITABLE.iter().find(|r| r.addr() == addr) {
            self.set(reg, byte);

            if reg == Reg::SLEEP_MODE && byte == SleepMode::DEEP_SLEEP {
                self.mode = PowerMode::DeepSleep;
            }
        }

        true
    }

    /// Reads the byte at the register pointer within a read transfer and advances the pointer.
    pub fn i2c_read(&mut self) -> u8 {
        let value = self.regs[usize::from(self.pointer)];
        self.pointer = self.pointer.wrapping_add(1);
        value
    }

    /// Puts a finger down at, or moves it to, the specified coordinates.
    ///
    /// Coordinates are truncated to 12 bits. Returns `true` if the chip pulses `INT`.
    pub fn touch(&mut self, x: u16, y: u16) -> bool {
        if self.mode == PowerMode::DeepSleep {
            return false;
        }

        self.mode = PowerMode::Dynamic;
        self.idle_ms = 0;

        let event = if self.touching {
            TouchEvent::Contact
        } else {
            TouchEvent::Down
        };

        let changed = !self.touching || self.position() != (x & 0x0FFF, y & 0x0FFF);
        self.touching = true;
        self.set_point(1, event, x, y);

        let irq = self.irq_ctl();
        irq.en_touch() || (irq.en_change() && changed)
    }

    /// Lifts the finger. Returns `true` if the chip pulses `INT`.
    pub fn release(&mut self) -> bool {
        if !self.touching || self.mode == PowerMode::DeepSleep {
            return false;
        }

        self.touching = false;
        self.idle_ms = 0;
        let (x, y) = self.position();
        self.set_point(0, TouchEvent::Up, x, y);

        let irq = self.irq_ctl();
        irq.en_touch() || irq.en_change()
    }

    /// Reports a gesture. Returns `true` if the chip pulses `INT`.
    ///
    /// A double click is only reported if enabled via [`MotionMask`].
    /// [`Gesture::Unknown`] is not a gesture the chip reports and is ignored.
    pub fn gesture(&mut self, gesture: Gesture) -> bool {
        if self.mode == PowerMode::DeepSleep || gesture == Gesture::Unknown {
            return false;
        }

        let mask = MotionMask::from(self.register(Reg::MOTION_MASK));

        if gesture == Gesture::DoubleClick && !mask.en_double_click() {
            return false;
        }

        self.mode = PowerMode::Dynamic;
        self.idle_ms = 0;
        self.set(Reg::GESTURE_ID, gesture.into_bits());
        self.irq_ctl().en_motion()
    }

    /// Advances time by `ms` milliseconds, entering standby when idle for long enough.
    pub fn tick(&mut self, ms: u32) {
        if self.mode != PowerMode::Dynamic || self.touching {
            return;
        }

        self.idle_ms = self.idle_ms.saturating_add(ms);
        let auto_sleep_enabled = self.register(Reg::DIS_AUTO_SLEEP) == 0;
        let timeout_ms = u32::from(self.register(Reg::AUTO_SLEEP_TIME)) * 1000;

        if auto_sleep_enabled && self.idle_ms >= timeout_ms {
            self.mode = PowerMode::Standby;
        }
    }

    fn position(&self) -> (u16, u16) {
        let x = u16::from(self.register(Reg::XPOS_H) & 0x0F) << 8
            | u16::from(self.register(Reg::XPOS_L));
        let y = u16::from(self.register(Reg::YPOS_H) & 0x0F) << 8
            | u16::from(self.register(Reg::YPOS_L));
        (x, y)
    }

    fn set_point(&mut self, fingers: u8, event: TouchEvent, x: u16, y: u16) {
        self.set(Reg::FINGER_NUM, fingers);
        self.set(
            Reg::XPOS_H,
            event.into_bits() << 6 | ((x >> 8) as u8 & 0x0F),
        );
        self.set(Reg::XPOS_L, x as u8);
        self.set(Reg::YPOS_H, (y >> 8) as u8 & 0x0F);
        self.set(Reg::YPOS_L, y as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(model: &mut Cst820Model, bytes: &[u8]) -> bool {
        model.i2c_start(false) && bytes.iter().all(|&b| model.i2c_write(b))
    }

    fn read<const N: usize>(model: &mut Cst820Model, reg: u8) -> [u8; N] {
        assert!(write(model, &[reg]));
        assert!(model.i2c_start(true));
        core::array::from_fn(|_| model.i2c_read())
    }

    #[test]
    fn reports_chip_id() {
        let mut model = Cst820Model::new();
        assert_eq!(read::<1>(&mut model, 0xA7), [CST820_CHIP_ID]);
    }

    #[test]
    fn touch_sequence() {
        let mut model = Cst820Model::new();
        assert!(model.touch(0x123, 0x0AB));
        assert_eq!(read::<5>(&mut model, 0x02), [1, 0x01, 0x23, 0x00, 0xAB]);

        assert!(model.touch(0x124, 0x0AB));
        assert_eq!(read::<2>(&mut model, 0x03), [0x80 | 0x01, 0x24]);

        assert!(model.release());
        assert_eq!(read::<2>(&mut model, 0x02), [0, 0x40 | 0x01]);
    }

    #[test]
    fn change_only_interrupts() {
        let mut model = Cst820Model::new();
        assert!(write(&mut model, &[0xFA, 0x20]));
        assert!(model.touch(10, 10));
        assert!(!model.touch(10, 10));
        assert!(model.touch(11, 10));
    }

    #[test]
    fn read_only_registers_ignore_writes() {
        let mut model = Cst820Model::new();
        assert!(write(&mut model, &[0xA7, 0x00]));
        assert_eq!(model.register(Reg::CHIP_ID), CST820_CHIP_ID);
    }

    #[test]
    fn multi_byte_write_auto_increments() {
        let mut model = Cst820Model::new();
        assert!(write(&mut model, &[0xF9, 5, 0x70]));
        assert_eq!(model.register(Reg::AUTO_SLEEP_TIME), 5);
        assert_eq!(model.register(Reg::IRQ_CTL), 0x70);
    }

    #[test]
    fn gestures() {
        let mut model = Cst820Model::new();
        assert!(write(&mut model, &[0xFA, 0x10]));
        assert!(model.gesture(Gesture::SlideLeft));
        assert_eq!(read::<1>(&mut model, 0x01), [0x03]);

        assert!(!model.gesture(Gesture::DoubleClick));
        assert!(!model.gesture(Gesture::Unknown));
        assert_eq!(read::<1>(&mut model, 0x01), [0x03]);

        assert!(write(&mut model, &[0xEC, 0x01]));
        assert!(model.gesture(Gesture::DoubleClick));
    }

    #[test]
    fn standby_after_idle_and_wake_on_touch() {
        let mut model = Cst820Model::new();
        model.tick(1999);
        assert_eq!(model.power_mode(), PowerMode::Dynamic);
        model.tick(1);
        assert_eq!(model.power_mode(), PowerMode::Standby);
        assert!(!model.i2c_start(false));

        model.touch(1, 1);
        assert_eq!(model.power_mode(), PowerMode::Dynamic);
        assert!(model.i2c_start(false));
    }

    #[test]
    fn disabled_auto_sleep() {
        let mut model = Cst820Model::new();
        assert!(write(&mut model, &[0xFE, 0x01]));
        model.tick(60_000);
        assert_eq!(model.power_mode(), PowerMode::Dynamic);
    }

    #[test]
    fn deep_sleep_until_reset() {
        let mut model = Cst820Model::new();
        assert!(write(&mut model, &[0xE5, 0x03]));
        assert_eq!(model.power_mode(), PowerMode::DeepSleep);
        assert!(!model.touch(1, 1));
        assert!(!model.i2c_start(true));

        model.reset();
        assert!(model.i2c_start(true));
    }
}
