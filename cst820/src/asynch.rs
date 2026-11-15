//! Async driver on `embedded-hal-async`.
//!
//! Mirrors the blocking [`Cst820`](crate::Cst820) and [`reset`](crate::reset).

driver!(
    doc = "Async CST820 driver.",
    i2c = embedded_hal_async::i2c::I2c,
    delay = embedded_hal_async::delay::DelayNs,
    async = [async],
    await = [.await],
);
