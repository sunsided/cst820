//! Wokwi custom chip wrapping the `cst820-sim` behavioral model.
//!
//! Pins: `VCC`, `GND`, `SDA`, `SCL`, `INT` (active-low pulses), `RST` (active low).
//!
//! Controls (also settable as attributes and via `set-control` in scenarios):
//! `touchX`, `touchY`, `touchPressed` (0/1) and `gesture` (an index into [`GESTURES`]).
//! The controls are sampled every [`SCAN_PERIOD_US`], like the chip's quick-scan period.

use cst820_registers::Gesture;
use cst820_sim::Cst820Model;
use std::cell::RefCell;
use std::ffi::{CString, c_void};
use wokwi_chip_ll::*;

/// The gestures selectable via the `gesture` control, by index.
const GESTURES: [Gesture; 8] = [
    Gesture::None,
    Gesture::SlideUp,
    Gesture::SlideDown,
    Gesture::SlideLeft,
    Gesture::SlideRight,
    Gesture::SingleClick,
    Gesture::DoubleClick,
    Gesture::LongPress,
];

/// How often the touch controls are sampled, in microseconds.
const SCAN_PERIOD_US: u32 = 10_000;

struct Chip {
    model: Cst820Model,
    int_pin: PinId,
    in_reset: bool,
    pulse_timer: TimerId,
    touch_x: AttrId,
    touch_y: AttrId,
    touch_pressed: AttrId,
    gesture: AttrId,
    was_pressed: bool,
    last_gesture: u32,
}

thread_local! {
    static CHIP: RefCell<Option<Chip>> = const { RefCell::new(None) };
}

fn with_chip<R>(f: impl FnOnce(&mut Chip) -> R) -> R {
    CHIP.with(|chip| f(chip.borrow_mut().as_mut().expect("chipInit not called")))
}

fn cstr(s: &str) -> CString {
    CString::new(s).expect("no interior NUL")
}

fn debug(message: &str) {
    let message = cstr(message);
    // SAFETY: `message` is a valid NUL-terminated string for the duration of the call.
    unsafe { debugPrint(message.as_ptr()) };
}

impl Chip {
    fn pulse_int(&mut self) {
        // SAFETY: Pin and timer IDs were obtained from the simulator in `chipInit`.
        unsafe {
            pinWrite(self.int_pin, LOW);
            timerStart(self.pulse_timer, self.model.irq_pulse_width_us(), false);
        }
    }

    fn scan(&mut self) {
        if self.in_reset {
            return;
        }
        // SAFETY: Attribute IDs were obtained from the simulator in `chipInit`.
        let (x, y, pressed, gesture) = unsafe {
            (
                attrRead(self.touch_x),
                attrRead(self.touch_y),
                attrRead(self.touch_pressed) != 0,
                attrRead(self.gesture),
            )
        };

        let mut irq = false;

        if pressed {
            irq |= self.model.touch(x as u16, y as u16);
        } else if self.was_pressed {
            irq |= self.model.release();
        }

        self.was_pressed = pressed;

        if gesture != self.last_gesture {
            let selected = GESTURES.get(gesture as usize).copied();

            if let Some(selected) = selected.filter(|&g| g != Gesture::None) {
                irq |= self.model.gesture(selected);
            }
        }

        self.last_gesture = gesture;

        self.model.tick(SCAN_PERIOD_US / 1000);

        if irq {
            self.pulse_int();
        }
    }
}

extern "C" fn on_connect(_: *mut c_void, _address: u32, is_read: bool) -> bool {
    with_chip(|chip| !chip.in_reset && chip.model.i2c_start(is_read))
}

extern "C" fn on_read(_: *mut c_void) -> u8 {
    with_chip(|chip| chip.model.i2c_read())
}

extern "C" fn on_write(_: *mut c_void, data: u8) -> bool {
    with_chip(|chip| chip.model.i2c_write(data))
}

extern "C" fn on_disconnect(_: *mut c_void) {}

extern "C" fn on_reset_change(_: *mut c_void, _pin: PinId, value: u32) {
    with_chip(|chip| {
        // shortcut: the chip answers right after RST rises; the real one needs >= 100 ms (Tron).
        chip.in_reset = value == LOW;

        if !chip.in_reset {
            chip.model.reset();
            chip.was_pressed = false;
            debug("cst820: reset");
        }
    });
}

extern "C" fn on_pulse_end(_: *mut c_void) {
    // SAFETY: The pin ID was obtained from the simulator in `chipInit`.
    with_chip(|chip| unsafe { pinWrite(chip.int_pin, HIGH) });
}

extern "C" fn on_scan(_: *mut c_void) {
    with_chip(Chip::scan);
}

/// Entry point called by the simulator.
#[unsafe(no_mangle)]
pub extern "C" fn chipInit() {
    // SAFETY: All pointers passed to the simulator are valid for the duration of each
    // call; configs are copied by the simulator. Init functions are only called here.
    unsafe {
        let int_pin = pinInit(cstr("INT").as_ptr(), OUTPUT_HIGH);
        let rst_pin = pinInit(cstr("RST").as_ptr(), INPUT_PULLUP);
        let sda = pinInit(cstr("SDA").as_ptr(), INPUT);
        let scl = pinInit(cstr("SCL").as_ptr(), INPUT);

        let model = Cst820Model::new();
        i2cInit(&I2CConfig {
            user_data: std::ptr::null(),
            address: u32::from(model.address()),
            scl,
            sda,
            connect: on_connect,
            read: on_read,
            write: on_write,
            disconnect: on_disconnect,
        });

        pinWatch(
            rst_pin,
            &WatchConfig {
                user_data: std::ptr::null(),
                edge: BOTH,
                pin_change: on_reset_change,
            },
        );

        let pulse_timer = timerInit(&TimerConfig {
            user_data: std::ptr::null(),
            callback: on_pulse_end,
        });
        let scan_timer = timerInit(&TimerConfig {
            user_data: std::ptr::null(),
            callback: on_scan,
        });

        let chip = Chip {
            model,
            int_pin,
            in_reset: pinRead(rst_pin) == LOW,
            pulse_timer,
            touch_x: attrInit(cstr("touchX").as_ptr(), 0.0),
            touch_y: attrInit(cstr("touchY").as_ptr(), 0.0),
            touch_pressed: attrInit(cstr("touchPressed").as_ptr(), 0.0),
            gesture: attrInit(cstr("gesture").as_ptr(), 0.0),
            was_pressed: false,
            last_gesture: 0,
        };

        CHIP.with(|c| *c.borrow_mut() = Some(chip));

        timerStart(scan_timer, SCAN_PERIOD_US, true);
    }

    debug("cst820: chip initialized");
}
