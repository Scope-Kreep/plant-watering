#pragma once

#include <Arduino.h>

#include "moisture.h"
#include "pump.h"

class Telemetry {
public:
  explicit Telemetry(Serial_ &out);

  void begin();
  void measurement(const MoistureSensor::Reading &soil) const;
  void state(const char *name, const char *reason = nullptr) const;
  void driver(const Pump::Diag &d, const char *error = nullptr) const;

private:
  Serial_ &out_;
};
