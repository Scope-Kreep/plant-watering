#pragma once

#include <Arduino.h>
#include <TMCStepper.h>

class Pump {
public:
  struct Reading {
    bool success;
    uint16_t currentScaleMin;
    uint16_t currentScaleMax;
    uint16_t stallGuardMin;
    uint16_t stallGuardMax;
    uint8_t pwmScale;
    uint16_t microstepCounter;
    uint32_t stepTime;
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
