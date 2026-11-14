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
  out_.print(F("{\"type\":\"moisture\",\"raw\":"));
  out_.print(soil.raw);
  out_.print(F(",\"spread\":"));
  out_.print(soil.spread);
  out_.print(F(",\"percent\":"));
  out_.print(soil.percent, 1);
  out_.print(F("}\n"));
}

void Telemetry::measurement(const Pump::Reading &stepper) const {
  out_.print(F("{\"type\":\"stepper\",\"pwm_scale\":"));
  out_.print(stepper.pwmScale);

  out_.print(F(",\"microstep_counter\":"));
  out_.print(stepper.microstepCounter);

  out_.print(F(",\"step_time\":"));
  out_.print(stepper.stepTime);

  out_.print(F(",\"current_scale\":"));
  out_.print(stepper.currentScale);

  out_.print(F(",\"sg_samples\":["));

  for (uint8_t i = 0; i < stepper.sgSampleCount; ++i) {
    if (i > 0)
      out_.print(',');

    out_.print(stepper.sgSamples[i]);
  }

  out_.print(']');

  out_.print(F("}\n"));
}

void Telemetry::state(const char *name, const char *reason) const {
  out_.print(F("{\"type\":\"state\",\"name\":\""));
  out_.print(name);
  out_.print('"');
  if (reason != nullptr) {
    out_.print(F(",\"reason\":\""));
    out_.print(reason);
    out_.print('"');
  }
  out_.print(F("}\n"));
}
