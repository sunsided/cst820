# CST820 on ESP32-C3

Reads touches from a CST820 with the blocking `cst820` driver and prints them over
the serial console.

| ESP32-C3 | CST820 |
|----------|--------|
| GPIO4    | SDA    |
| GPIO5    | SCL    |
| GPIO6    | RST    |
| GPIO7    | INT    |

## Run in Wokwi

Requires `rustup target add riscv32imc-unknown-none-elf wasm32-unknown-unknown`,
[`wokwi-cli`](https://docs.wokwi.com/wokwi-ci/cli-installation) and `WOKWI_CLI_TOKEN`.

```sh
../../sim/wokwi-chip/build.sh
cargo build --release
wokwi-cli . --scenario scenarios/touch.yaml --timeout 10000
```

## Run on hardware

```sh
cargo run --release   # uses espflash
```

## Register dump (for hardware verification)

`src/bin/register-dump.rs` answers the open questions about the real chip
([#1](https://github.com/sunsided/cst820/issues/1),
[#2](https://github.com/sunsided/cst820/issues/2),
[#3](https://github.com/sunsided/cst820/issues/3)). It does not configure the chip, and
uses SDA = GPIO4, SCL = GPIO5, RST = GPIO6 (adjust the pins for your board).

```sh
cargo run --release --bin register-dump
```

1. Leave the panel untouched for a few seconds: the `config` lines show the power-on
   defaults, and a `read failed` line about 2 s after boot shows standby NACKing.
2. Touch, move and lift one finger.
3. Touch with two fingers, move them, lift one, then the other.
4. Paste the log into the issue.

## Adding another board

Create `examples/<board>/` with its own firmware, `wokwi.toml` and `diagram.json`,
and reference the same chip binary (`../../sim/wokwi-chip/cst820.chip.wasm`).
