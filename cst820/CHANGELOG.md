# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- [**breaking**] `read_register` requires `ReadableRegister`, and `modify_register`
  requires a register that is both readable and writable (`cst820-registers` 0.2,
  `hardware-registers` 0.4).

## [0.1.0] - 2026-11-15

[0.1.0]: https://github.com/sunsided/cst820/releases/tag/v0.1.0

### Added

- Blocking driver on `embedded-hal` 1.0 and async driver on `embedded-hal-async` 1.0
  (`async` feature): touch readout, chip identification, project and firmware
  version, gesture, interrupt and auto-sleep
  configuration, deep sleep, and hardware reset.
