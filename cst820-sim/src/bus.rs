//! [`fake_bus`] adapter for the [`Cst820Model`].

use crate::Cst820Model;
use core::cell::RefCell;
use fake_bus::I2cTarget;

/// An I²C bus with a single [`Cst820Model`] attached.
///
/// The model is shared through a [`RefCell`] so that tests can inject touches while a
/// driver owns the bus. For a bus shared with other models, use [`fake_bus::FakeI2c`]
/// with an array of targets.
pub type FakeI2c<'a> = fake_bus::FakeI2c<&'a RefCell<Cst820Model>>;

impl I2cTarget for Cst820Model {
    fn address(&self) -> u8 {
        Cst820Model::address(self)
    }

    fn start(&mut self, is_read: bool) -> bool {
        self.i2c_start(is_read)
    }

    fn write(&mut self, byte: u8) -> bool {
        self.i2c_write(byte)
    }

    fn read(&mut self) -> u8 {
        self.i2c_read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake_bus::embedded_hal::i2c::{ErrorKind, I2c, NoAcknowledgeSource, Operation};

    #[test]
    fn write_read() {
        let model = RefCell::new(Cst820Model::new());
        let mut bus = FakeI2c::new(&model);
        let mut id = [0];
        bus.write_read(0x15, &[0xA7], &mut id).unwrap();
        assert_eq!(id, [0xB7]);
    }

    #[test]
    fn adjacent_writes_form_one_transfer() {
        let model = RefCell::new(Cst820Model::new());
        let mut bus = FakeI2c::new(&model);
        bus.transaction(
            0x15,
            &mut [Operation::Write(&[0xF9]), Operation::Write(&[7])],
        )
        .unwrap();
        assert_eq!(model.borrow().register(crate::Reg::AUTO_SLEEP_TIME), 7);
    }

    #[test]
    fn wrong_address_nacks() {
        let model = RefCell::new(Cst820Model::new());
        let mut bus = FakeI2c::new(&model);
        assert_eq!(
            bus.write(0x16, &[0xA7]),
            Err(ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address))
        );
    }

    #[test]
    fn standby_nacks() {
        let model = RefCell::new(Cst820Model::new());
        let mut bus = FakeI2c::new(&model);
        model.borrow_mut().tick(2_000);
        assert!(bus.write(0x15, &[0xA7]).is_err());
    }

    #[test]
    fn async_write_read() {
        let model = RefCell::new(Cst820Model::new());
        let mut bus = FakeI2c::new(&model);
        let mut id = [0];
        embassy_futures::block_on(fake_bus::embedded_hal_async::i2c::I2c::write_read(
            &mut bus,
            0x15,
            &[0xA7],
            &mut id,
        ))
        .unwrap();
        assert_eq!(id, [0xB7]);
    }
}
