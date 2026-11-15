//! An `embedded-hal` I²C bus backed by a [`Cst820Model`].

use crate::Cst820Model;
use core::cell::RefCell;
use embedded_hal::i2c::{ErrorKind, ErrorType, NoAcknowledgeSource, Operation, SevenBitAddress};

/// An I²C bus with a single [`Cst820Model`] attached.
///
/// The model is shared through a [`RefCell`] so that tests can inject touches while a
/// driver owns the bus.
#[derive(Debug, Clone, Copy)]
pub struct FakeI2c<'a> {
    model: &'a RefCell<Cst820Model>,
}

impl<'a> FakeI2c<'a> {
    /// Creates a bus with the specified model attached.
    pub fn new(model: &'a RefCell<Cst820Model>) -> Self {
        Self { model }
    }

    fn run(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), ErrorKind> {
        let mut model = self.model.borrow_mut();

        if address != model.address() {
            return Err(ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address));
        }

        // Adjacent operations of the same type form one transfer (no repeated start).
        let mut previous_is_read = None;

        for operation in operations {
            let is_read = matches!(operation, Operation::Read(_));

            if previous_is_read != Some(is_read) && !model.i2c_start(is_read) {
                return Err(ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address));
            }

            previous_is_read = Some(is_read);

            match operation {
                Operation::Write(bytes) => {
                    if !bytes.iter().all(|&b| model.i2c_write(b)) {
                        return Err(ErrorKind::NoAcknowledge(NoAcknowledgeSource::Data));
                    }
                }

                Operation::Read(buffer) => {
                    buffer.iter_mut().for_each(|b| *b = model.i2c_read());
                }
            }
        }

        Ok(())
    }
}

impl ErrorType for FakeI2c<'_> {
    type Error = ErrorKind;
}

impl embedded_hal::i2c::I2c<SevenBitAddress> for FakeI2c<'_> {
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.run(address, operations)
    }
}

impl embedded_hal_async::i2c::I2c<SevenBitAddress> for FakeI2c<'_> {
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.run(address, operations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal::i2c::I2c;

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
    fn adjacent_reads_continue() {
        let model = RefCell::new(Cst820Model::new());
        let mut bus = FakeI2c::new(&model);
        let (mut a, mut b) = ([0], [0]);
        bus.transaction(
            0x15,
            &mut [
                Operation::Write(&[0xA7]),
                Operation::Read(&mut a),
                Operation::Read(&mut b),
            ],
        )
        .unwrap();
        assert_eq!((a, b), ([0xB7], [0x00]));
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
    fn async_write_read() {
        let model = RefCell::new(Cst820Model::new());
        let mut bus = FakeI2c::new(&model);
        let mut id = [0];
        embassy_futures::block_on(embedded_hal_async::i2c::I2c::write_read(
            &mut bus,
            0x15,
            &[0xA7],
            &mut id,
        ))
        .unwrap();
        assert_eq!(id, [0xB7]);
    }
}
