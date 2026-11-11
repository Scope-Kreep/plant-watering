#pragma once

#include <Arduino.h>
#include <TMCStepper.h>

namespace {
static constexpr uint8_t kMaxLoadSamples = 320;
}

class Pump {
public:
  struct Reading {
    bool success;
    uint16_t currentScale;
    uint8_t pwmScale;
    uint16_t microstepCounter;
    uint32_t stepTime;
    uint16_t sgSamples[kMaxLoadSamples] = {};
    uint16_t sgSampleCount = 0;
  };

  Pump(HardwareSerial &uart);

  bool begin();

  Reading pulse();

  const char *error() const { return error_; }

private:
  void configure();
  void run(long steps, Reading &result);
  bool driverFaulted();

  HardwareSerial &uart_;
  TMC2209Stepper driver_;
  const char *error_ = nullptr;
};
