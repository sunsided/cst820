# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Dropped `#![deny(warnings)]`, so a warning added by a newer compiler can no longer
  fail the docs.rs build.

## [0.2.2] - 2026-11-15

[0.2.2]: https://github.com/sunsided/cst820/releases/tag/v0.2.2

### Changed

- With the `defmt` feature, registers print their fields (e.g.
  `IrqCtl { en_test: false, en_touch: true, … }`) instead of the raw byte.

## [0.2.1] - 2026-11-15

[0.2.1]: https://github.com/sunsided/cst820/releases/tag/v0.2.1

No changes; released alongside `cst820` 0.2.1, which shares the version.

## [0.2.0] - 2026-11-15

[0.2.0]: https://github.com/sunsided/cst820/releases/tag/v0.2.0

### Changed

- [**breaking**] Updated `hardware-registers` to 0.4. `Register` is now the base trait;
  reading requires `ReadableRegister` and writing requires `WritableRegister`.
- [**breaking**] `SleepMode` (`0xE5`) is write-only: it is a command register and can no
  longer be read through the typed API.

## [0.1.0] - 2026-11-15

[0.1.0]: https://github.com/sunsided/cst820/releases/tag/v0.1.0

### Added

- Initial register map, following the CST816S register declaration.
