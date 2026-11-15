# Hynitron CST820 Touch Controller Registers

[![Crates.io][crates-image]][crates-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Safety Dance][safety-image]][safety-link]
![MSRV][msrv-image]
[![EUPL 1.2 licensed][license-eupl-image]][license-eupl-link]
[![Apache 2.0 licensed][license-apache-image]][license-apache-link]
[![MIT licensed][license-mit-image]][license-mit-link]

> A typed map of the CST820's I²C-accessible registers.

Provides functions and named fields to read and modify the controller registers,
built on [`hardware-registers`](https://crates.io/crates/hardware-registers).

The CST820 datasheet (V1.2) does not document the register map. The map in this crate
follows the CST816S register declaration, a member of the same chip family. The touch
event flags in `XposH[7:6]` and the sleep register at `0xE5` are not part of that
document either; they follow vendor drivers.

## Sources

| Fact | Sources |
|------|---------|
| Register map `0x01`–`0x06`, `0xA7`–`0xA9`, `0xB0`–`0xB3`, `0xEC`–`0xFE` | CST816S register declaration (Hynitron) |
| Chip ID `0xB7` | [LuatOS `luat_tp_cst820.c`][luatos], [Realtek `touch_cst820.c`][realtek], [Zephyr `input_cst8xx.c`][zephyr], [SensorLib][sensorlib], ESPHome |
| Event bits `XposH[7:6]` (0 down, 1 up, 2 contact, 3 none) | [Zephyr][zephyr], [LuatOS][luatos], [Realtek][realtek] |
| Touch ID in `YposH[7:4]` | [LuatOS][luatos] |
| Deep sleep: write `0x03` to `0xE5`, wake only by reset | [Zephyr][zephyr], [SensorLib][sensorlib]; reset wake: CST816S/CST816D datasheets |
| `IrqCtl` bits | CST816S register declaration, [Zephyr][zephyr]; CST820 modes `0x60`/`0x11`/`0x71`: [Actions SDK][actions] |
| No I²C response in auto-sleep until touched | [SensorLib author on the CST820][nack] |

No source documents a second touch point or the power-on value of `IrqCtl`; see
[#1](https://github.com/sunsided/cst820/issues/1) and
[#3](https://github.com/sunsided/cst820/issues/3).

[luatos]: https://github.com/openLuat/LuatOS/blob/master/components/tp/luat_tp_cst820.c
[realtek]: https://github.com/realmcu/wearable/blob/main/device/touch/8762D/touch_cst820.c
[zephyr]: https://github.com/zephyrproject-rtos/zephyr/blob/main/drivers/input/input_cst8xx.c
[sensorlib]: https://github.com/lewisxhe/SensorLib/tree/master/src/touch
[actions]: https://github.com/lvgl/lv_port_actions_technology/blob/master/action_technology_sdk/zephyr/drivers/input/tpkey/cst820_tpkey_acts.c
[nack]: https://github.com/Xinyuan-LilyGO/LilyGo-T-RGB/issues/43#issuecomment-1899513862

See also the [`cst820`](https://crates.io/crates/cst820) driver.

## License

Copyright © 2026 Markus Mayer

Triple licensed under your choice of either of:

- European Union Public Licence, Version 1.2, ([LICENSE-EUPL](LICENSE-EUPL)
  or https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12)
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

[crates-image]: https://img.shields.io/crates/v/cst820-registers
[crates-link]: https://crates.io/crates/cst820-registers
[docs-image]: https://docs.rs/cst820-registers/badge.svg
[docs-link]: https://docs.rs/cst820-registers/
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
