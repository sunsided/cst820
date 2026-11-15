# CST820 Wokwi custom chip

Wraps the [`cst820-sim`](../../cst820-sim) behavioral model into a
[Wokwi custom chip](https://docs.wokwi.com/chips-api/getting-started).

**[Live demo on wokwi.com](https://wokwi.com/projects/480766558097043457)**

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

Projects outside this repository (on wokwi.com, in VS Code or with `wokwi-cli`) can use
the `chip.zip` attached to each GitHub Release instead of building the chip. Add a
`chip-cst820` part and a dependency to `diagram.json`; no `[[chip]]` entry is needed:

```json
"dependencies": { "chip-cst820": "github:sunsided/cst820@0.1.0" }
```

The version is given without the `v` prefix of the release tag; `@v0.1.0` fails to load.

| Pin   | Function                         |
|-------|----------------------------------|
| `SDA` | I²C data (address `0x15`)        |
| `SCL` | I²C clock                        |
| `INT` | Interrupt output, active-low pulses |
| `RST` | Reset input, active low          |

Controls `touchX`, `touchY` (0–4095), `touchPressed` (0/1) and `gesture` are sampled every
10 ms. `gesture` selects one of: 0 none, 1 slide up, 2 slide down, 3 slide left, 4 slide
right, 5 single click, 6 double click (needs `MotionMask`), 7 long press. Each change of
the control reports one gesture. Drive the controls from the Wokwi UI or with
`set-control` in automation scenarios.

The chip answers immediately after `RST` rises; the real chip needs at least 100 ms.
