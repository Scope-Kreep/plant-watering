#include "pump.h"
#include <Arduino.h>

Pump pump(Serial1);

void setup() {
  Serial.begin(115200);

  if (!pump.begin()) {
    // TODO: FAIL
  }
}

void loop() {}
