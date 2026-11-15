# Hynitron CST820 Touch Controller Driver

[![Crates.io][crates-image]][crates-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Safety Dance][safety-image]][safety-link]
![MSRV][msrv-image]
[![EUPL 1.2 licensed][license-eupl-image]][license-eupl-link]
[![Apache 2.0 licensed][license-apache-image]][license-apache-link]
[![MIT licensed][license-mit-image]][license-mit-link]

> A platform-agnostic driver to interface with the Hynitron CST820 touch controller.

Built on [`embedded-hal`] 1.0 and the [`cst820-registers`] map. Enable the `async`
feature for a driver on [`embedded-hal-async`].

```rust,ignore
use cst820::{Cst820, reset};

reset(&mut rst_pin, &mut delay)?;
let mut touch = Cst820::new(i2c);
assert!(touch.identify()?);

// Wait for the INT pin to go low, then:
let t = touch.read_touch()?;
if t.pressed() {
    println!("{} {} {:?}", t.x, t.y, t.event);
}
```

## Implemented so far

- Touch readout (coordinates, finger count, event, gesture) in a single burst read
- Chip identification
- Interrupt, auto-sleep and deep sleep configuration
- Hardware reset with datasheet timing
- Direct access to registers via [`cst820-registers`] structs and [`hardware-registers`] traits

The chip enters standby after 2 s without touch and then does not acknowledge on the bus.
Disable auto-sleep via `set_auto_sleep(None)` right after reset if you need to poll.

[`embedded-hal`]: https://crates.io/crates/embedded-hal
[`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async
[`cst820-registers`]: https://crates.io/crates/cst820-registers
[`hardware-registers`]: https://crates.io/crates/hardware-registers

## License

Copyright © 2026 Markus Mayer

Triple licensed under your choice of either of:

- European Union Public Licence, Version 1.2, ([LICENSE-EUPL](LICENSE-EUPL)
  or https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12)
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

[crates-image]: https://img.shields.io/crates/v/cst820
[crates-link]: https://crates.io/crates/cst820
[docs-image]: https://docs.rs/cst820/badge.svg
[docs-link]: https://docs.rs/cst820/
[build-image]: https://github.com/sunsided/cst820/workflows/Rust/badge.svg
[build-link]: https://github.com/sunsided/cst820/actions
[safety-image]: https://img.shields.io/badge/unsafe-forbidden-success.svg
[safety-link]: https://github.com/rust-secure-code/safety-dance/
[msrv-image]: https://img.shields.io/badge/rustc-1.85+-blue.svg
[license-eupl-image]: https://img.shields.io/badge/license-EUPL_1.2-blue.svg
[license-eupl-link]: https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12
[license-apache-image]: https://img.shields.io/badge/license-Apache_2.0-blue.svg
[license-apache-link]: https://www.apache.org/licenses/LICENSE-2.0
[license-mit-image]: https://img.shields.io/badge/license-MIT-blue.svg
[license-mit-link]: https://opensource.org/licenses/MIT
