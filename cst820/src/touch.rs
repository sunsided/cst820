//! Decoding of the touch registers.

use cst820_registers::{
    FingerNum, Gesture, GestureId, RegisterAddress, TouchEvent, XposH, XposL, YposH, YposL,
};

/// A snapshot of the touch registers.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Touch {
    /// The X coordinate (12 bits).
    pub x: u16,
    /// The Y coordinate (12 bits).
    pub y: u16,
    /// The number of fingers on the panel.
    pub fingers: u8,
    /// The touch event.
    pub event: TouchEvent,
    /// The most recent gesture.
    pub gesture: Gesture,
}

impl Touch {
    /// The first register of the burst read.
    pub(crate) const START: RegisterAddress = RegisterAddress::GESTURE_ID;

    /// The number of bytes in the burst read (`GestureID` through `YposL`).
    pub(crate) const LEN: usize = 6;

    /// Decodes the registers `GestureID` (01h) through `YposL` (06h).
    pub fn from_bytes(bytes: &[u8; Self::LEN]) -> Self {
        let gesture = GestureId::from(bytes[0]);
        let fingers = FingerNum::from(bytes[1]);
        let xh = XposH::from(bytes[2]);
        let xl = XposL::from(bytes[3]);
        let yh = YposH::from(bytes[4]);
        let yl = YposL::from(bytes[5]);
        Self {
            x: u16::from(xh.xpos_high()) << 8 | u16::from(xl.value()),
            y: u16::from(yh.ypos_high()) << 8 | u16::from(yl.value()),
            fingers: fingers.value(),
            event: xh.event(),
            gesture: gesture.gesture(),
        }
    }

    /// Whether a finger is on the panel.
    ///
    /// Requires a plausible finger count and a down or contact event, so that invalid
    /// data (e.g. all registers reading `0xFF`) and lift events do not count as touches.
    pub fn pressed(&self) -> bool {
        matches!(self.fingers, 1..=2)
            && matches!(self.event, TouchEvent::Down | TouchEvent::Contact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode() {
        let touch = Touch::from_bytes(&[0x04, 0x01, 0x81, 0x23, 0xF0, 0xAB]);
        assert_eq!(
            touch,
            Touch {
                x: 0x123,
                y: 0x0AB,
                fingers: 1,
                event: TouchEvent::Contact,
                gesture: Gesture::SlideRight,
            }
        );
        assert!(touch.pressed());
    }

    #[test]
    fn garbage_is_not_a_touch() {
        // Some CST8xx chips return 0xFF for every register, e.g. right after auto-sleep
        // is turned off.
        assert!(!Touch::from_bytes(&[0xFF; Touch::LEN]).pressed());
    }

    #[test]
    fn lift_is_not_a_touch() {
        assert!(!Touch::from_bytes(&[0x00, 0x01, 0x40, 0x10, 0x00, 0x10]).pressed());
    }
}
