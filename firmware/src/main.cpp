#include "controller.h"
#include "moisture.h"
#include "pump.h"
#include "telemetry.h"
#include <Arduino.h>

MoistureSensor soil(A3);
Pump pump(Serial1);
Telemetry telemetry(Serial);
Controller controller(soil, pump, telemetry);

void setup() { controller.begin(); }

void loop() { controller.update(); }
