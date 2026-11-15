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

## Adding another board

Create `examples/<board>/` with its own firmware, `wokwi.toml` and `diagram.json`,
and reference the same chip binary (`../../sim/wokwi-chip/cst820.chip.wasm`).
