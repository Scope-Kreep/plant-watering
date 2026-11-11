#include "moisture.h"
#include "pump.h"
#include <Arduino.h>

MoistureSensor moisture(A3);
Pump pump(Serial1);

void setup() {
  Serial.begin(115200);

  if (!pump.begin()) {
    // TODO: FAIL
  }
}

void loop() {
  const auto m = moisture.read();
  Serial.print(m.raw);
  Serial.print(",");
  Serial.print(m.percent, 1);
  Serial.print(",");
  Serial.println(m.spread);
}
