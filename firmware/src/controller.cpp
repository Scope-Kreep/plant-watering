#include "controller.h"

namespace {
constexpr uint32_t kMeasureIntervalMs = 15UL * 60UL * 1000UL;
}

Controller::Controller(MoistureSensor &soil, Pump &pump, Telemetry &telemetry)
    : soil_(soil), pump_(pump), telemetry_(telemetry) {};

void Controller::begin() {
  telemetry_.begin();
  telemetry_.state(name(state_));

  nextMeasureMs_ = millis();

  const bool pumpOk = pump_.begin();
  telemetry_.driver(pump_.diagnostics(), pumpOk ? nullptr : pump_.error());
  if (!pumpOk) {
    enter(State::Fault, pump_.error());
    return;
  }

  enter(State::Idle);
}

void Controller::update() {
  switch (state_) {
  case State::Boot:
    break;

  case State::Idle:
    if (measurementDue()) {
      enter(State::Measure);
    }
    break;

<<<<<<< Updated upstream
  case State::Measure:
=======
  case State::Measure: {
    // temp: kjør pumpen ved hver måling for testing
    const bool pumpOk = pump_.pulse();
    telemetry_.driver(pump_.diagnostics(), pumpOk ? nullptr : pump_.error());
>>>>>>> Stashed changes
    takeMeasurement();
    enter(State::Idle);
    break;
  }

  case State::Fault:
    if (measurementDue()) {
      takeMeasurement();
    }
    break;
  }
}

void Controller::enter(State next, const char *reason) {
  if (next == state_) {
    return;
  }

  state_ = next;
  telemetry_.state(name(next), reason);
}

void Controller::takeMeasurement() {
  telemetry_.measurement(soil_.read());
  nextMeasureMs_ += kMeasureIntervalMs;
}

bool Controller::measurementDue() const {
  return static_cast<int32_t>(millis() - nextMeasureMs_) >= 0;
}

const char *Controller::name(State state) {
  switch (state) {
  case State::Boot:
    return "BOOT";
  case State::Idle:
    return "IDLE";
  case State::Measure:
    return "MEASURE";
  case State::Fault:
    return "FAULT";
  }

  return "UNKNOWN";
}
