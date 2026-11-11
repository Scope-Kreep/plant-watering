#pragma once

#include "moisture.h"
#include <Arduino.h>

class Telemetry {
public:
  Telemetry(Serial_ &out);

  void begin();

  void measurement(const MoistureSensor::Reading &moisture) const;

  void state(const char *name, const char *reason = nullptr) const;

private:
  Serial_ &out_;
};
