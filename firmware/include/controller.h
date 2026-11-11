#pragma once

#include "moisture.h"
#include "pump.h"
#include "telemetry.h"
#include <Arduino.h>

class Controller {
public:
  Controller(MoistureSensor &soil, Pump &pump, Telemetry &telemetry);

  void begin();

  void update();

private:
  enum class State : uint8_t { Boot, Idle, Measure, Fault };

  void enter(State next, const char *reason = nullptr);
  void takeMeasurement();
  bool measurementDue() const;
  static const char *name(State state);

  MoistureSensor &soil_;
  Pump &pump_;
  Telemetry &telemetry_;

  State state_ = State::Boot;
  uint32_t nextMeasureMs_ = 0;
};
