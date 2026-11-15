//! Value types used in register fields.

/// A detected gesture, as reported by [`GestureId`](crate::GestureId).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Gesture {
    /// No gesture.
    None = 0x00,
    /// Slide up.
    SlideUp = 0x01,
    /// Slide down.
    SlideDown = 0x02,
    /// Slide left.
    SlideLeft = 0x03,
    /// Slide right.
    SlideRight = 0x04,
    /// Single click.
    SingleClick = 0x05,
    /// Double click. Must be enabled via [`MotionMask::en_double_click`](crate::MotionMask).
    DoubleClick = 0x0B,
    /// Long press.
    LongPress = 0x0C,
    /// A value not covered by the register declaration.
    Unknown = 0xFF,
}

impl Gesture {
    /// Converts the value into its register representation.
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    /// Converts a register value into a gesture.
    pub const fn from_bits(value: u8) -> Self {
        match value {
            0x00 => Self::None,
            0x01 => Self::SlideUp,
            0x02 => Self::SlideDown,
            0x03 => Self::SlideLeft,
            0x04 => Self::SlideRight,
            0x05 => Self::SingleClick,
            0x0B => Self::DoubleClick,
            0x0C => Self::LongPress,
            _ => Self::Unknown,
        }
    }
}

/// The touch event, as reported in the upper bits of [`XposH`](crate::XposH).
///
/// These bits are not part of the CST816S register declaration; the encoding follows
/// vendor drivers for the CST8xx family.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum TouchEvent {
    /// A finger was put down.
    Down = 0b00,
    /// A finger was lifted.
    Up = 0b01,
    /// A finger stays in contact.
    Contact = 0b10,
    /// No event.
    None = 0b11,
}

impl TouchEvent {
    /// Converts the value into its register representation.
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    /// Converts the two lowest bits of a register value into a touch event.
    pub const fn from_bits(value: u8) -> Self {
        match value & 0b11 {
            0b00 => Self::Down,
            0b01 => Self::Up,
            0b10 => Self::Contact,
            _ => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gesture_roundtrip() {
        for g in [
            Gesture::None,
            Gesture::SlideUp,
            Gesture::SlideDown,
            Gesture::SlideLeft,
            Gesture::SlideRight,
            Gesture::SingleClick,
            Gesture::DoubleClick,
            Gesture::LongPress,
        ] {
            assert_eq!(Gesture::from_bits(g.into_bits()), g);
        }

        assert_eq!(Gesture::from_bits(0x42), Gesture::Unknown);
    }

    #[test]
    fn touch_event_roundtrip() {
        for e in [
            TouchEvent::Down,
            TouchEvent::Up,
            TouchEvent::Contact,
            TouchEvent::None,
        ] {
            assert_eq!(TouchEvent::from_bits(e.into_bits()), e);
        }
    }
}
