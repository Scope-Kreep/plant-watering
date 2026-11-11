#include "telemetry.h"

namespace {
constexpr unsigned long kUSBBaud = 115200;

void flag(Serial_ &out, const __FlashStringHelper *key, bool value) {
  out.print(F(",\""));
  out.print(key);
  out.print(F("\":"));
  out.print(value ? 1 : 0);
}
} // namespace

Telemetry::Telemetry(Serial_ &out) : out_(out) {}

void Telemetry::begin() {
  out_.begin(kUSBBaud);

  const uint32_t start = millis();
  while (!out_ && millis() - start < 3000) {
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

void Telemetry::driver(const Pump::Diag &d, const char *error) const {
  const uint32_t s = d.drvStatus;

  out_.print(F("{\"type\":\"driver\",\"version\":"));
  out_.print(d.version);
  out_.print(F(",\"microsteps\":"));
  out_.print(d.microsteps);

  flag(out_, F("reconfigured"), d.reconfigured);
  flag(out_, F("reset"), d.gstat & 0x01);
  flag(out_, F("drv_err"), d.gstat & 0x02);
  flag(out_, F("uv_cp"), d.gstat & 0x04);
  flag(out_, F("otpw"), s & (1UL << 0));
  flag(out_, F("ot"), s & (1UL << 1));
  flag(out_, F("short"), s & (0xFUL << 2));
  flag(out_, F("open_a"), s & (1UL << 6));
  flag(out_, F("open_b"), s & (1UL << 7));
  flag(out_, F("stealth"), s & (1UL << 30));

  out_.print(F(",\"samples\":"));
  out_.print(d.samples);
  if (d.samples == 0) {
    out_.print(F(",\"sg\":null,\"cs\":null"));
  } else {
    out_.print(F(",\"sg\":["));
    out_.print(d.sgMin);
    out_.print(',');
    out_.print(d.sgMax);
    out_.print(F("],\"cs\":["));
    out_.print(d.csMin);
    out_.print(',');
    out_.print(d.csMax);
    out_.print(']');
  }

  if (error != nullptr) {
    out_.print(F(",\"error\":\""));
    out_.print(error);
    out_.print('"');
  }

  out_.print(F("}\n"));
}
