//! Controller registers.
//!
//! Register addresses, names, bit layouts and defaults follow the CST816S register
//! declaration unless noted otherwise.

use crate::types::{Gesture, TouchEvent};
use bitfield_struct::bitfield;

/// Register addresses.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum RegisterAddress {
    /// See [`GestureId`]. Read-only.
    GESTURE_ID = 0x01,
    /// See [`FingerNum`]. Read-only.
    FINGER_NUM = 0x02,
    /// See [`XposH`]. Read-only.
    XPOS_H = 0x03,
    /// See [`XposL`]. Read-only.
    XPOS_L = 0x04,
    /// See [`YposH`]. Read-only.
    YPOS_H = 0x05,
    /// See [`YposL`]. Read-only.
    YPOS_L = 0x06,
    /// See [`ChipId`]. Read-only.
    CHIP_ID = 0xA7,
    /// See [`ProjId`]. Read-only.
    PROJ_ID = 0xA8,
    /// See [`FwVersion`]. Read-only.
    FW_VERSION = 0xA9,
    /// See [`Bpc0H`]. Read-only.
    BPC0_H = 0xB0,
    /// See [`Bpc0L`]. Read-only.
    BPC0_L = 0xB1,
    /// See [`Bpc1H`]. Read-only.
    BPC1_H = 0xB2,
    /// See [`Bpc1L`]. Read-only.
    BPC1_L = 0xB3,
    /// See [`SleepMode`]. Write-only.
    SLEEP_MODE = 0xE5,
    /// See [`MotionMask`]. Read-write.
    MOTION_MASK = 0xEC,
    /// See [`IrqPulseWidth`]. Read-write.
    IRQ_PULSE_WIDTH = 0xED,
    /// See [`NorScanPer`]. Read-write.
    NOR_SCAN_PER = 0xEE,
    /// See [`MotionSlAngle`]. Read-write.
    MOTION_SL_ANGLE = 0xEF,
    /// See [`LpScanRaw1H`]. Read-only.
    LP_SCAN_RAW1_H = 0xF0,
    /// See [`LpScanRaw1L`]. Read-only.
    LP_SCAN_RAW1_L = 0xF1,
    /// See [`LpScanRaw2H`]. Read-only.
    LP_SCAN_RAW2_H = 0xF2,
    /// See [`LpScanRaw2L`]. Read-only.
    LP_SCAN_RAW2_L = 0xF3,
    /// See [`LpAutoWakeTime`]. Read-write.
    LP_AUTO_WAKE_TIME = 0xF4,
    /// See [`LpScanTh`]. Read-write.
    LP_SCAN_TH = 0xF5,
    /// See [`LpScanWin`]. Read-write.
    LP_SCAN_WIN = 0xF6,
    /// See [`LpScanFreq`]. Read-write.
    LP_SCAN_FREQ = 0xF7,
    /// See [`LpScanIdac`]. Read-write.
    LP_SCAN_IDAC = 0xF8,
    /// See [`AutoSleepTime`]. Read-write.
    AUTO_SLEEP_TIME = 0xF9,
    /// See [`IrqCtl`]. Read-write.
    IRQ_CTL = 0xFA,
    /// See [`AutoReset`]. Read-write.
    AUTO_RESET = 0xFB,
    /// See [`LongPressTime`]. Read-write.
    LONG_PRESS_TIME = 0xFC,
    /// See [`IoCtl`]. Read-write.
    IO_CTL = 0xFD,
    /// See [`DisAutoSleep`]. Read-write.
    DIS_AUTO_SLEEP = 0xFE,
}

impl RegisterAddress {
    /// Returns the address of a register.
    pub const fn addr(&self) -> u8 {
        *self as u8
    }
}

impl From<RegisterAddress> for u8 {
    fn from(value: RegisterAddress) -> Self {
        value.addr()
    }
}

/// Declares a register whose eight bits form a single value.
macro_rules! value_register {
    ($(#[$meta:meta])* $type:ident, $addr:ident, $access:ident, $writable:ident) => {
        value_register!($(#[$meta])* $type, $addr, $access, $writable, 0);
    };

    ($(#[$meta:meta])* $type:ident, $addr:ident, $access:ident, $writable:ident, $default:expr) => {
        $(#[$meta])*
        #[bitfield(u8, order = Msb)]
        #[derive(PartialEq, Eq)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct $type {
            /// The register value.
            #[bits(8, access = $access, default = $default)]
            pub value: u8,
        }

        $writable!($type, RegisterAddress::$addr);
    };
}

/// [`GESTURE_ID`](RegisterAddress::GESTURE_ID) (01h)
#[bitfield(u8, order = Msb)]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GestureId {
    /// The detected gesture.
    #[bits(8, access = RO)]
    pub gesture: Gesture,
}

readable_register!(GestureId, RegisterAddress::GESTURE_ID);

value_register!(
    /// [`FINGER_NUM`](RegisterAddress::FINGER_NUM) (02h)
    ///
    /// The number of fingers: `0` for no finger, `1` for one finger.
    FingerNum, FINGER_NUM, RO, readable_register
);

/// [`XPOS_H`](RegisterAddress::XPOS_H) (03h)
#[bitfield(u8, order = Msb)]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct XposH {
    /// The touch event.
    ///
    /// Not part of the CST816S register declaration; see [`TouchEvent`].
    #[bits(2, access = RO)]
    pub event: TouchEvent,

    #[bits(2)]
    __: u8,

    /// The high 4 bits of the X coordinate.
    #[bits(4, access = RO)]
    pub xpos_high: u8,
}

readable_register!(XposH, RegisterAddress::XPOS_H);

value_register!(
    /// [`XPOS_L`](RegisterAddress::XPOS_L) (04h)
    ///
    /// The low 8 bits of the X coordinate.
    XposL, XPOS_L, RO, readable_register
);

/// [`YPOS_H`](RegisterAddress::YPOS_H) (05h)
#[bitfield(u8, order = Msb)]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct YposH {
    /// Touch ID bits as reported by vendor drivers; undocumented.
    #[bits(4, access = RO)]
    pub touch_id: u8,

    /// The high 4 bits of the Y coordinate.
    #[bits(4, access = RO)]
    pub ypos_high: u8,
}

readable_register!(YposH, RegisterAddress::YPOS_H);

value_register!(
    /// [`YPOS_L`](RegisterAddress::YPOS_L) (06h)
    ///
    /// The low 8 bits of the Y coordinate.
    YposL, YPOS_L, RO, readable_register
);

value_register!(
    /// [`CHIP_ID`](RegisterAddress::CHIP_ID) (A7h)
    ///
    /// The chip ID. See [`CST820_CHIP_ID`](crate::CST820_CHIP_ID).
    ChipId, CHIP_ID, RO, readable_register
);

value_register!(
    /// [`PROJ_ID`](RegisterAddress::PROJ_ID) (A8h)
    ///
    /// The project ID.
    ProjId, PROJ_ID, RO, readable_register
);

value_register!(
    /// [`FW_VERSION`](RegisterAddress::FW_VERSION) (A9h)
    ///
    /// The firmware version.
    FwVersion, FW_VERSION, RO, readable_register
);

value_register!(
    /// [`BPC0_H`](RegisterAddress::BPC0_H) (B0h)
    ///
    /// The high 8 bits of the BPC0 value.
    Bpc0H, BPC0_H, RO, readable_register
);

value_register!(
    /// [`BPC0_L`](RegisterAddress::BPC0_L) (B1h)
    ///
    /// The low 8 bits of the BPC0 value.
    Bpc0L, BPC0_L, RO, readable_register
);

value_register!(
    /// [`BPC1_H`](RegisterAddress::BPC1_H) (B2h)
    ///
    /// The high 8 bits of the BPC1 value.
    Bpc1H, BPC1_H, RO, readable_register
);

value_register!(
    /// [`BPC1_L`](RegisterAddress::BPC1_L) (B3h)
    ///
    /// The low 8 bits of the BPC1 value.
    Bpc1L, BPC1_L, RO, readable_register
);

value_register!(
    /// [`SLEEP_MODE`](RegisterAddress::SLEEP_MODE) (E5h)
    ///
    /// Writing [`SleepMode::DEEP_SLEEP`] puts the chip into deep sleep; only a reset
    /// via the `RST` pin wakes it up again.
    ///
    /// Not part of the CST816S register declaration; follows vendor drivers.
    SleepMode, SLEEP_MODE, RW, writable_register
);

impl SleepMode {
    /// The value that enters deep sleep.
    pub const DEEP_SLEEP: u8 = 0x03;

    /// Creates a register value that enters deep sleep.
    pub const fn deep_sleep() -> Self {
        Self::new().with_value(Self::DEEP_SLEEP)
    }
}

/// [`MOTION_MASK`](RegisterAddress::MOTION_MASK) (ECh)
#[bitfield(u8, order = Msb)]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MotionMask {
    #[bits(5)]
    __: u8,

    /// Enable continuous left-right scrolling action.
    #[bits(1, access = RW)]
    pub en_continuous_lr: bool,

    /// Enable continuous up-down scrolling action.
    #[bits(1, access = RW)]
    pub en_continuous_ud: bool,

    /// Enable the double click action.
    #[bits(1, access = RW)]
    pub en_double_click: bool,
}

writable_register!(MotionMask, RegisterAddress::MOTION_MASK);

value_register!(
    /// [`IRQ_PULSE_WIDTH`](RegisterAddress::IRQ_PULSE_WIDTH) (EDh)
    ///
    /// Interrupt low-pulse output width. Unit: 0.1 ms, range: 1 to 200, default: 10.
    IrqPulseWidth, IRQ_PULSE_WIDTH, RW, writable_register, 10
);

value_register!(
    /// [`NOR_SCAN_PER`](RegisterAddress::NOR_SCAN_PER) (EEh)
    ///
    /// Normal quick-scanning period. Affects [`LpAutoWakeTime`] and [`AutoSleepTime`].
    /// Unit: 10 ms, range: 1 to 30, default: 1.
    NorScanPer, NOR_SCAN_PER, RW, writable_register, 1
);

value_register!(
    /// [`MOTION_SL_ANGLE`](RegisterAddress::MOTION_SL_ANGLE) (EFh)
    ///
    /// Gesture detection sliding area angle control: `tan(c) * 10`, where `c` is the
    /// angle with respect to the positive x-axis.
    MotionSlAngle, MOTION_SL_ANGLE, RW, writable_register
);

value_register!(
    /// [`LP_SCAN_RAW1_H`](RegisterAddress::LP_SCAN_RAW1_H) (F0h)
    ///
    /// The high 8 bits of the low-power scanning channel 1 reference value.
    LpScanRaw1H, LP_SCAN_RAW1_H, RO, readable_register
);

value_register!(
    /// [`LP_SCAN_RAW1_L`](RegisterAddress::LP_SCAN_RAW1_L) (F1h)
    ///
    /// The low 8 bits of the low-power scanning channel 1 reference value.
    LpScanRaw1L, LP_SCAN_RAW1_L, RO, readable_register
);

value_register!(
    /// [`LP_SCAN_RAW2_H`](RegisterAddress::LP_SCAN_RAW2_H) (F2h)
    ///
    /// The high 8 bits of the low-power scanning channel 2 reference value.
    LpScanRaw2H, LP_SCAN_RAW2_H, RO, readable_register
);

value_register!(
    /// [`LP_SCAN_RAW2_L`](RegisterAddress::LP_SCAN_RAW2_L) (F3h)
    ///
    /// The low 8 bits of the low-power scanning channel 2 reference value.
    LpScanRaw2L, LP_SCAN_RAW2_L, RO, readable_register
);

value_register!(
    /// [`LP_AUTO_WAKE_TIME`](RegisterAddress::LP_AUTO_WAKE_TIME) (F4h)
    ///
    /// Automatic recalibration period during low-power mode.
    /// Unit: 1 minute, range: 1 to 5, default: 5.
    LpAutoWakeTime, LP_AUTO_WAKE_TIME, RW, writable_register, 5
);

value_register!(
    /// [`LP_SCAN_TH`](RegisterAddress::LP_SCAN_TH) (F5h)
    ///
    /// Low-power scanning wake-up threshold; smaller is more sensitive.
    /// Range: 1 to 255, default: 48.
    LpScanTh, LP_SCAN_TH, RW, writable_register, 48
);

value_register!(
    /// [`LP_SCAN_WIN`](RegisterAddress::LP_SCAN_WIN) (F6h)
    ///
    /// Low-power scanning measurement range; greater is more sensitive and draws more power.
    /// Range: 0 to 3, default: 3.
    LpScanWin, LP_SCAN_WIN, RW, writable_register, 3
);

value_register!(
    /// [`LP_SCAN_FREQ`](RegisterAddress::LP_SCAN_FREQ) (F7h)
    ///
    /// Low-power scanning frequency; smaller is more sensitive. Range: 1 to 255, default: 7.
    LpScanFreq, LP_SCAN_FREQ, RW, writable_register, 7
);

value_register!(
    /// [`LP_SCAN_IDAC`](RegisterAddress::LP_SCAN_IDAC) (F8h)
    ///
    /// Low-power scanning current; smaller is more sensitive. Range: 1 to 255.
    LpScanIdac, LP_SCAN_IDAC, RW, writable_register
);

value_register!(
    /// [`AUTO_SLEEP_TIME`](RegisterAddress::AUTO_SLEEP_TIME) (F9h)
    ///
    /// Enter low-power (standby) mode after this many seconds without touch.
    /// Unit: 1 s, default: 2.
    AutoSleepTime, AUTO_SLEEP_TIME, RW, writable_register, 2
);

/// [`IRQ_CTL`](RegisterAddress::IRQ_CTL) (FAh)
#[bitfield(u8, order = Msb)]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IrqCtl {
    /// Interrupt pin test: periodically generates low pulses.
    #[bits(1, access = RW)]
    pub en_test: bool,

    /// Generate low pulses when a touch is detected.
    #[bits(1, access = RW)]
    pub en_touch: bool,

    /// Generate low pulses when the touch changes.
    #[bits(1, access = RW)]
    pub en_change: bool,

    /// Generate low pulses when a gesture is detected.
    #[bits(1, access = RW)]
    pub en_motion: bool,

    #[bits(3)]
    __: u8,

    /// Generate only one low pulse on a long press gesture.
    #[bits(1, access = RW)]
    pub once_wlp: bool,
}

writable_register!(IrqCtl, RegisterAddress::IRQ_CTL);

value_register!(
    /// [`AUTO_RESET`](RegisterAddress::AUTO_RESET) (FBh)
    ///
    /// Automatically reset if there is a touch but no valid gesture within this many
    /// seconds. Unit: 1 s, `0` disables.
    AutoReset, AUTO_RESET, RW, writable_register
);

value_register!(
    /// [`LONG_PRESS_TIME`](RegisterAddress::LONG_PRESS_TIME) (FCh)
    ///
    /// Automatically reset after a long press of this many seconds.
    /// Unit: 1 s, `0` disables, default: 10.
    LongPressTime, LONG_PRESS_TIME, RW, writable_register, 10
);

/// [`IO_CTL`](RegisterAddress::IO_CTL) (FDh)
#[bitfield(u8, order = Msb)]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IoCtl {
    #[bits(5)]
    __: u8,

    /// Enable soft reset: the host resets the touch controller by pulling the IRQ pin low.
    #[bits(1, access = RW)]
    pub soft_reset: bool,

    /// I²C pin driver mode: `false` for the internal pull-up resistor (default),
    /// `true` for open drain.
    #[bits(1, access = RW)]
    pub i2c_open_drain: bool,

    /// I²C and IRQ pin level: `false` for VDD (default), `true` for 1.8 V.
    #[bits(1, access = RW)]
    pub en_1v8: bool,
}

writable_register!(IoCtl, RegisterAddress::IO_CTL);

value_register!(
    /// [`DIS_AUTO_SLEEP`](RegisterAddress::DIS_AUTO_SLEEP) (FEh)
    ///
    /// `0` (default) enables automatic entry into low-power mode; any non-zero value
    /// disables it.
    DisAutoSleep, DIS_AUTO_SLEEP, RW, writable_register
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn xpos_high_layout() {
        let reg = XposH::from(0b1001_0101);
        assert_eq!(reg.event(), TouchEvent::Contact);
        assert_eq!(reg.xpos_high(), 0b0101);
    }

    #[test]
    fn ypos_high_layout() {
        let reg = YposH::from(0b0001_1010);
        assert_eq!(reg.touch_id(), 0b0001);
        assert_eq!(reg.ypos_high(), 0b1010);
    }

    #[test]
    fn gesture_id_layout() {
        assert_eq!(GestureId::from(0x0B).gesture(), Gesture::DoubleClick);
    }

    #[test]
    fn irq_ctl_layout() {
        let reg = IrqCtl::new()
            .with_en_touch(true)
            .with_en_change(true)
            .with_en_motion(true);
        assert_eq!(u8::from(reg), 0x70);
        assert_eq!(u8::from(IrqCtl::new().with_en_test(true)), 0x80);
        assert_eq!(u8::from(IrqCtl::new().with_once_wlp(true)), 0x01);
    }

    #[test]
    fn motion_mask_layout() {
        assert_eq!(
            u8::from(MotionMask::new().with_en_continuous_lr(true)),
            0b100
        );
        assert_eq!(
            u8::from(MotionMask::new().with_en_continuous_ud(true)),
            0b010
        );
        assert_eq!(
            u8::from(MotionMask::new().with_en_double_click(true)),
            0b001
        );
    }

    #[test]
    fn io_ctl_layout() {
        assert_eq!(u8::from(IoCtl::new().with_soft_reset(true)), 0b100);
        assert_eq!(u8::from(IoCtl::new().with_i2c_open_drain(true)), 0b010);
        assert_eq!(u8::from(IoCtl::new().with_en_1v8(true)), 0b001);
    }

    #[test]
    fn documented_defaults() {
        assert_eq!(IrqPulseWidth::new().value(), 10);
        assert_eq!(NorScanPer::new().value(), 1);
        assert_eq!(LpAutoWakeTime::new().value(), 5);
        assert_eq!(LpScanTh::new().value(), 48);
        assert_eq!(LpScanWin::new().value(), 3);
        assert_eq!(LpScanFreq::new().value(), 7);
        assert_eq!(AutoSleepTime::new().value(), 2);
        assert_eq!(LongPressTime::new().value(), 10);
    }

    #[test]
    fn sleep_mode_value() {
        assert_eq!(u8::from(SleepMode::deep_sleep()), 0x03);
    }

    #[test]
    fn register_addresses() {
        assert_eq!(*ChipId::REGISTER_ADDRESS, 0xA7);
        assert_eq!(*IrqCtl::REGISTER_ADDRESS, 0xFA);
        assert_eq!(*ChipId::DEFAULT_DEVICE_ADDRESS, 0x15);
    }
}
