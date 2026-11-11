#include "telemetry.h"

namespace {
constexpr unsigned long kUSBBaud = 115200;
}

Telemetry::Telemetry(Serial_ &out) : out_(out) {}

void Telemetry::begin() {
  out_.begin(kUSBBaud);

  const uint32_t start = millis();
  while (!Serial && millis() - start < 3000) {
  }
}

void Telemetry::measurement(const MoistureSensor::Reading &soil) const {
  out_.print(F("{\"type\":\"measurement\",\"soil_raw\":"));
  out_.print(soil.raw);
  out_.print(F(",\"soil_spread\":"));
  out_.print(soil.spread);
  out_.print(F(",\"soil_percent\":"));
  out_.print(soil.percent, 1);
  out_.print(F("}\n"));
}

void Telemetry::measurement(const Pump::Reading &stepper) const {
  out_.print(F("{\"type\":\"measurement\",\"stepper_actual_current_scale\":"));
  out_.print(stepper.actualCurrentScale);
  out_.print(F(",\"stepper_stall_guard_result\":"));
  out_.print(stepper.stallGuardResult);
  out_.print(F(",\"stepper_pwm_scale\":"));
  out_.print(stepper.pwmScale);
  out_.print(F(",\"stepper_microstep_counter\":"));
  out_.print(stepper.microstepCounter);
  out_.print(F(",\"stepper_step_time\":"));
  out_.print(stepper.stepTime);
  out_.print(F("}\n"));
}

void Telemetry::state(const char *name, const char *reason) const {
  out_.print(F("{\"type\":\"state\",\"state\":\""));
  out_.print(name);
  out_.print('"');
  if (reason != nullptr) {
    out_.print(F(",\"reason\":\""));
    out_.print(reason);
    out_.print('"');
  }
  out_.print(F("}\n"));
}
