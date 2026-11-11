#include "moisture.h"

namespace {
constexpr uint8_t kSamples = 11;
constexpr uint16_t kSampleGapMs = 5;

constexpr uint16_t kRawDry = 450;
constexpr uint16_t kRawWet = 145;
} // namespace

MoistureSensor::MoistureSensor(uint8_t pin) : pin_(pin) {}

MoistureSensor::Reading MoistureSensor::read() const {
  uint16_t samples[kSamples];

  analogRead(pin_);

  for (uint8_t i = 0; i < kSamples; ++i) {
    samples[i] = analogRead(pin_);
    delay(kSampleGapMs);
  }

  for (uint8_t i = 1; i < kSamples; ++i) {
    const uint16_t value = samples[i];
    int8_t j = i - 1;
    while (j >= 0 && samples[j] > value) {
      samples[j + 1] = samples[j];
      --j;
    }
    samples[j + 1] = value;
  }

  const uint16_t median = samples[kSamples / 2];
  const uint16_t spread = samples[kSamples - 1] - samples[0];
  return {median, spread, toPercent(median)};
}

float MoistureSensor::toPercent(uint16_t raw) {
  const float percent =
      100.0f * (float(kRawDry) - float(raw)) / float(kRawDry - kRawWet);

  return constrain(percent, 0.0f, 100.0f);
}
