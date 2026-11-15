//! Driver tests against the `cst820-sim` behavioral model.

use core::cell::RefCell;
use cst820::registers::{AutoSleepTime, IrqCtl, RegisterAddress as Reg};
use cst820::{Cst820, Gesture, TouchEvent};
use cst820_sim::{Cst820Model, FakeI2c, PowerMode};
use embedded_hal::i2c::{ErrorKind, NoAcknowledgeSource};

#[test]
fn identifies_chip() {
    let model = RefCell::new(Cst820Model::new());
    let mut driver = Cst820::new(FakeI2c::new(&model));
    assert!(driver.identify().unwrap());
}

#[test]
fn wrong_address_fails() {
    let model = RefCell::new(Cst820Model::new());
    let mut driver = Cst820::with_address(FakeI2c::new(&model), 0x16);
    assert_eq!(
        driver.identify(),
        Err(ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address))
    );
}

#[test]
fn reads_touch_lifecycle() {
    let model = RefCell::new(Cst820Model::new());
    let mut driver = Cst820::new(FakeI2c::new(&model));

    let idle = driver.read_touch().unwrap();
    assert!(!idle.pressed());
    assert_eq!(idle.event, TouchEvent::None);

    assert!(model.borrow_mut().touch(120, 80));
    let down = driver.read_touch().unwrap();
    assert_eq!((down.x, down.y, down.fingers), (120, 80, 1));
    assert_eq!(down.event, TouchEvent::Down);

    model.borrow_mut().touch(300, 1000);
    let moved = driver.read_touch().unwrap();
    assert_eq!((moved.x, moved.y), (300, 1000));
    assert_eq!(moved.event, TouchEvent::Contact);

    model.borrow_mut().release();
    let up = driver.read_touch().unwrap();
    assert!(!up.pressed());
    assert_eq!(up.event, TouchEvent::Up);
}

#[test]
fn reads_gestures() {
    let model = RefCell::new(Cst820Model::new());
    let mut driver = Cst820::new(FakeI2c::new(&model));
    driver
        .configure_interrupts(IrqCtl::new().with_en_motion(true))
        .unwrap();
    assert!(model.borrow_mut().gesture(Gesture::SlideUp));
    assert_eq!(driver.read_touch().unwrap().gesture, Gesture::SlideUp);
}

#[test]
fn auto_sleep_configuration() {
    let model = RefCell::new(Cst820Model::new());
    let mut driver = Cst820::new(FakeI2c::new(&model));

    driver.set_auto_sleep(Some(5)).unwrap();
    assert_eq!(driver.read_register::<AutoSleepTime>().unwrap().value(), 5);
    model.borrow_mut().tick(5_000);
    assert_eq!(model.borrow().power_mode(), PowerMode::Standby);
    assert!(driver.identify().is_err());

    model.borrow_mut().reset();
    driver.set_auto_sleep(None).unwrap();
    model.borrow_mut().tick(60_000);
    assert!(driver.identify().unwrap());
}

#[test]
fn modify_register() {
    let model = RefCell::new(Cst820Model::new());
    let mut driver = Cst820::new(FakeI2c::new(&model));
    driver
        .modify_register(|r: IrqCtl| r.with_en_motion(true))
        .unwrap();
    assert_eq!(model.borrow().register(Reg::IRQ_CTL), 0x70);
}

#[test]
fn deep_sleep() {
    let model = RefCell::new(Cst820Model::new());
    let mut driver = Cst820::new(FakeI2c::new(&model));
    driver.sleep().unwrap();
    assert_eq!(model.borrow().power_mode(), PowerMode::DeepSleep);
    assert!(driver.identify().is_err());
}

#[test]
fn reset_timing() {
    use embedded_hal::delay::DelayNs;
    use embedded_hal::digital::{ErrorType, OutputPin};

    #[derive(Default)]
    struct Log {
        steps: Vec<String>,
    }

    struct Pin<'a>(&'a RefCell<Log>);
    struct Delay<'a>(&'a RefCell<Log>);

    impl ErrorType for Pin<'_> {
        type Error = core::convert::Infallible;
    }

    impl OutputPin for Pin<'_> {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.0.borrow_mut().steps.push("low".into());
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.0.borrow_mut().steps.push("high".into());
            Ok(())
        }
    }

    impl DelayNs for Delay<'_> {
        fn delay_ns(&mut self, ns: u32) {
            self.0.borrow_mut().steps.push(format!("{ns}ns"));
        }
    }

    let log = RefCell::new(Log::default());
    cst820::reset(&mut Pin(&log), &mut Delay(&log)).unwrap();
    let steps = log.into_inner().steps;
    // Default `delay_us`/`delay_ms` split into `delay_ns` calls; sum them per phase.
    let low = steps.iter().position(|s| s == "low").unwrap();
    let high = steps.iter().position(|s| s == "high").unwrap();
    let total_ns = |range: &[String]| -> u64 {
        range
            .iter()
            .map(|s| s.trim_end_matches("ns").parse::<u64>().unwrap())
            .sum()
    };

    assert_eq!(low, 0);
    assert!(total_ns(&steps[low + 1..high]) >= 100_000);
    assert!(total_ns(&steps[high + 1..]) >= 100_000_000);
}

#[cfg(feature = "async")]
mod asynch {
    use super::*;
    use embassy_futures::block_on;

    #[test]
    fn identifies_and_reads_touch() {
        let model = RefCell::new(Cst820Model::new());
        let mut driver = cst820::asynch::Cst820::new(FakeI2c::new(&model));
        assert!(block_on(driver.identify()).unwrap());

        model.borrow_mut().touch(42, 7);
        let touch = block_on(driver.read_touch()).unwrap();
        assert_eq!((touch.x, touch.y, touch.event), (42, 7, TouchEvent::Down));
    }

    #[test]
    fn configures_auto_sleep() {
        let model = RefCell::new(Cst820Model::new());
        let mut driver = cst820::asynch::Cst820::new(FakeI2c::new(&model));
        block_on(driver.set_auto_sleep(None)).unwrap();
        model.borrow_mut().tick(60_000);
        assert!(block_on(driver.identify()).unwrap());
    }

    #[test]
    fn reads_versions_and_gestures() {
        use cst820::registers::MotionMask;

        let model = RefCell::new(Cst820Model::new());
        let mut driver = cst820::asynch::Cst820::with_address(FakeI2c::new(&model), 0x15);
        assert_eq!(
            block_on(driver.firmware_version()).unwrap(),
            cst820_sim::MODEL_FW_VERSION
        );
        assert_eq!(block_on(driver.project_id()).unwrap(), 0);

        block_on(driver.configure_interrupts(IrqCtl::new().with_en_motion(true))).unwrap();
        block_on(driver.configure_gestures(MotionMask::new().with_en_double_click(true))).unwrap();
        assert!(model.borrow_mut().gesture(Gesture::DoubleClick));
        assert_eq!(
            block_on(driver.read_touch()).unwrap().gesture,
            Gesture::DoubleClick
        );
    }

    #[test]
    fn modifies_registers_and_sleeps() {
        let model = RefCell::new(Cst820Model::new());
        let mut driver = cst820::asynch::Cst820::new(FakeI2c::new(&model));
        block_on(driver.modify_register(|r: IrqCtl| r.with_en_motion(true))).unwrap();
        assert_eq!(model.borrow().register(Reg::IRQ_CTL), 0x70);

        block_on(driver.set_auto_sleep(Some(5))).unwrap();
        assert_eq!(
            block_on(driver.read_register::<AutoSleepTime>())
                .unwrap()
                .value(),
            5
        );

        block_on(driver.sleep()).unwrap();
        assert_eq!(model.borrow().power_mode(), PowerMode::DeepSleep);

        let bus = driver.release();
        model.borrow_mut().reset();
        assert!(Cst820::new(bus).identify().unwrap());
    }

    #[test]
    fn reset_timing() {
        use embedded_hal::digital::{ErrorType, OutputPin};
        use embedded_hal_async::delay::DelayNs;

        struct Pin<'a>(&'a RefCell<Vec<u64>>);
        struct Delay<'a>(&'a RefCell<Vec<u64>>);

        impl ErrorType for Pin<'_> {
            type Error = core::convert::Infallible;
        }

        impl OutputPin for Pin<'_> {
            fn set_low(&mut self) -> Result<(), Self::Error> {
                self.0.borrow_mut().push(0);
                Ok(())
            }

            fn set_high(&mut self) -> Result<(), Self::Error> {
                self.0.borrow_mut().push(0);
                Ok(())
            }
        }

        impl DelayNs for Delay<'_> {
            async fn delay_ns(&mut self, ns: u32) {
                self.0.borrow_mut().push(u64::from(ns));
            }
        }

        // Pin changes are logged as 0, delays as their duration in ns.
        let log = RefCell::new(Vec::new());
        block_on(cst820::asynch::reset(&mut Pin(&log), &mut Delay(&log))).unwrap();
        let log = log.into_inner();
        let mut phases = log.split(|&v| v == 0).skip(1);
        assert!(phases.next().unwrap().iter().sum::<u64>() >= 100_000);
        assert!(phases.next().unwrap().iter().sum::<u64>() >= 100_000_000);
    }
}

#[test]
fn reads_versions() {
    let model = RefCell::new(Cst820Model::new());
    let mut driver = Cst820::new(FakeI2c::new(&model));
    assert_eq!(
        driver.firmware_version().unwrap(),
        cst820_sim::MODEL_FW_VERSION
    );
    assert_eq!(driver.project_id().unwrap(), 0);
}

#[test]
fn double_click_requires_motion_mask() {
    use cst820::registers::MotionMask;

    let model = RefCell::new(Cst820Model::new());
    let mut driver = Cst820::new(FakeI2c::new(&model));
    driver
        .configure_interrupts(IrqCtl::new().with_en_motion(true))
        .unwrap();
    assert!(!model.borrow_mut().gesture(Gesture::DoubleClick));

    driver
        .configure_gestures(MotionMask::new().with_en_double_click(true))
        .unwrap();
    assert!(model.borrow_mut().gesture(Gesture::DoubleClick));
    assert_eq!(driver.read_touch().unwrap().gesture, Gesture::DoubleClick);
}
