// CST820 touch controller demo for the Wokwi web editor (Arduino, ESP32-C3).
//
// Talks to the CST820 Wokwi chip over raw I2C and prints touches in the same format as
// the Rust example in ../esp32c3. Wiring: SDA = GPIO4, SCL = GPIO5, RST = GPIO6, INT = GPIO7.

#include <Wire.h>

constexpr uint8_t CST820_ADDRESS = 0x15;
constexpr uint8_t CST820_CHIP_ID = 0xB7;

constexpr uint8_t REG_GESTURE_ID = 0x01;
constexpr uint8_t REG_CHIP_ID = 0xA7;
constexpr uint8_t REG_IRQ_CTL = 0xFA;

// IrqCtl: pulse INT on touch changes (EnChange) and gestures (EnMotion).
constexpr uint8_t IRQ_EN_CHANGE_MOTION = 0x30;

constexpr int PIN_SDA = 4;
constexpr int PIN_SCL = 5;
constexpr int PIN_RST = 6;
constexpr int PIN_INT = 7;

const char *eventName(uint8_t event) {
  switch (event) {
    case 0: return "Down";
    case 1: return "Up";
    case 2: return "Contact";
    default: return "None";
  }
}

const char *gestureName(uint8_t gesture) {
  switch (gesture) {
    case 0x00: return "None";
    case 0x01: return "SlideUp";
    case 0x02: return "SlideDown";
    case 0x03: return "SlideLeft";
    case 0x04: return "SlideRight";
    case 0x05: return "SingleClick";
    case 0x0B: return "DoubleClick";
    case 0x0C: return "LongPress";
    default: return "Unknown";
  }
}

bool readRegisters(uint8_t reg, uint8_t *buffer, size_t len) {
  Wire.beginTransmission(CST820_ADDRESS);
  Wire.write(reg);
  if (Wire.endTransmission(false) != 0) {
    return false;
  }

  if (Wire.requestFrom(int(CST820_ADDRESS), int(len)) != int(len)) {
    return false;
  }

  for (size_t i = 0; i < len; i++) {
    buffer[i] = Wire.read();
  }

  return true;
}

void writeRegister(uint8_t reg, uint8_t value) {
  Wire.beginTransmission(CST820_ADDRESS);
  Wire.write(reg);
  Wire.write(value);
  Wire.endTransmission();
}

void setup() {
  Serial.begin(115200);

  // Reset: RST low for at least 0.1 ms, then wait at least 100 ms (datasheet Trst, Tron).
  pinMode(PIN_RST, OUTPUT);
  digitalWrite(PIN_RST, LOW);
  delay(1);
  digitalWrite(PIN_RST, HIGH);
  delay(100);

  pinMode(PIN_INT, INPUT_PULLUP);
  Wire.begin(PIN_SDA, PIN_SCL);
  Wire.setClock(400000);

  uint8_t chipId = 0;
  if (!readRegisters(REG_CHIP_ID, &chipId, 1) || chipId != CST820_CHIP_ID) {
    Serial.printf("CST820 not found (chip ID 0x%02X)\n", chipId);
    while (true) {
      delay(1000);
    }
  }

  writeRegister(REG_IRQ_CTL, IRQ_EN_CHANGE_MOTION);
  Serial.printf("CST820 ok chip_id=0x%02X\n", chipId);
}

void loop() {
  // INT pulses low when there is something to read.
  while (digitalRead(PIN_INT) == HIGH) {
  }

  // GestureID, FingerNum, XposH, XposL, YposH, YposL in one burst.
  uint8_t data[6];
  if (readRegisters(REG_GESTURE_ID, data, sizeof(data))) {
    uint16_t x = (uint16_t(data[2] & 0x0F) << 8) | data[3];
    uint16_t y = (uint16_t(data[4] & 0x0F) << 8) | data[5];
    Serial.printf("touch x=%u y=%u fingers=%u ev=%s gesture=%s\n", x, y, data[1],
                  eventName(data[2] >> 6), gestureName(data[0]));
  } else {
    Serial.println("read failed");
  }

  while (digitalRead(PIN_INT) == LOW) {
  }
}
