#include "controller.h"

namespace {
// constexpr uint32_t kMeasureIntervalMs = 15UL * 60UL * 1000UL;
constexpr uint32_t kMeasureIntervalMs = 5UL * 1000UL;
} // namespace

Controller::Controller(MoistureSensor &soil, Pump &pump, Telemetry &telemetry)
    : soil_(soil), pump_(pump), telemetry_(telemetry) {};

void Controller::begin() {
  telemetry_.begin();
  telemetry_.state(name(state_));

  nextMeasureMs_ = millis();

  if (!pump_.begin()) {
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

  case State::Measure:
    // temp
    pump_.pulse();
    takeMeasurement();
    enter(State::Idle);
    break;

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
