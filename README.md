# Hynitron CST820 Touch Controller

> Platform-agnostic Rust support for the Hynitron CST820 self-capacitive touch controller.

| Crate                                    | Description                                              |
|------------------------------------------|----------------------------------------------------------|
| [`cst820-registers`](cst820-registers)   | A typed map of the CST820's I²C registers.               |
| [`cst820`](cst820)                       | An `embedded-hal` 1.0 driver (blocking and async).       |
| [`cst820-sim`](cst820-sim)               | A behavioral model of the chip for tests and emulation.  |

The CST820 datasheet (V1.2) documents the I²C protocol and timing, but not the register map.
The register map used here follows the CST816 family, which the CST820 is compatible with.

## Emulation

The behavioral model in `cst820-sim` is used both as a fake I²C bus in host tests and as a
[Wokwi](https://wokwi.com) custom chip (`sim/wokwi-chip`). Example firmware in `examples/`
runs against the emulated chip using `wokwi-cli`.

## Releasing

Both crates share one version. From `main`, with the changes listed under
`## [Unreleased]` in each crate's `CHANGELOG.md`:

```sh
cargo release <patch|minor|major>            # dry run
cargo release <patch|minor|major> --execute
```

This publishes `cst820-registers` and `cst820`, commits `Release X.Y.Z`, and pushes the tag
`vX.Y.Z`. The tag creates a GitHub Release named `X.Y.Z` with the changelog sections as notes.

## Code of Conduct

We abide by the [Contributor Covenant][cc] and ask that you do as well.

## License

Copyright © 2026 Markus Mayer

Triple licensed under your choice of either of:

- European Union Public Licence, Version 1.2, ([LICENSE-EUPL](LICENSE-EUPL)
  or https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12)
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

[cc]: https://contributor-covenant.org
