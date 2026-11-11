#include "moisture.h"
#include "pump.h"
#include "telemetry.h"
#include <Arduino.h>

MoistureSensor moisture(A3);
Pump pump(Serial1);
Telemetry telemetry(Serial);

void setup() {
  telemetry.begin();
  telemetry.state("BOOT");

  if (!pump.begin()) {
    telemetry.state("FAULT", pump.error());
    // TODO: go to fault state when FSM exists
  }

  telemetry.state("MEASURE");
}

void loop() {
  telemetry.measurement(moisture.read());
  delay(1000);
}
