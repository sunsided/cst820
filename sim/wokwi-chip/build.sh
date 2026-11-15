#!/usr/bin/env sh
# Builds the chip and places it next to cst820.chip.json, as Wokwi expects.
set -eu
cd "$(dirname "$0")"
cargo build --release
cp target/wasm32-unknown-unknown/release/cst820_wokwi_chip.wasm cst820.chip.wasm
