# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
