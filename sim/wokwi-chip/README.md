# CST820 Wokwi custom chip

Wraps the [`cst820-sim`](../../cst820-sim) behavioral model into a
[Wokwi custom chip](https://docs.wokwi.com/chips-api/getting-started).

```sh
./build.sh   # produces cst820.chip.wasm next to cst820.chip.json
```

Reference it from a project's `wokwi.toml`:

```toml
[[chip]]
name = 'cst820'
binary = '../../sim/wokwi-chip/cst820.chip.wasm'
```

and add a `chip-cst820` part to `diagram.json`.

Projects outside this repository can use the `chip.zip` attached to each GitHub Release
instead of building the chip, via `diagram.json` dependencies:

```json
"dependencies": { "chip-cst820": "github:sunsided/cst820@vX.Y.Z" }
```

| Pin   | Function                         |
|-------|----------------------------------|
| `SDA` | I²C data (address `0x15`)        |
| `SCL` | I²C clock                        |
| `INT` | Interrupt output, active-low pulses |
| `RST` | Reset input, active low          |

Controls `touchX`, `touchY` (0–4095), `touchPressed` (0/1) and `gesture` (gesture ID)
are sampled every 10 ms. Drive them from the Wokwi UI or with `set-control` in
automation scenarios.

The chip answers immediately after `RST` rises; the real chip needs at least 100 ms.
