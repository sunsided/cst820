# CST820 demo for the Wokwi web editor

wokwi.com no longer compiles Rust online, so this demo uses an Arduino sketch. It speaks
raw I²C to the same CST820 chip and prints touches in the same format as the
[Rust example](../esp32c3). The chip is loaded from this repository's GitHub Release via
the `dependencies` entry in `diagram.json`.

Live project: <https://wokwi.com/projects/480766558097043457>

To recreate it on wokwi.com:

1. Start a new **ESP32-C3** Arduino project.
2. Replace `sketch.ino` and `diagram.json` with the files from this directory.
3. Start the simulation. Click the CST820 chip and use its `touchX`, `touchY`,
   `touchPressed` and `gesture` controls.
