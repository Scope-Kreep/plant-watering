#pragma once

#include <Arduino.h>
#include <TMCStepper.h>

class Pump {
public:
  struct Reading {
    bool success;
    uint16_t actualCurrentScale;
    uint16_t stallGuardResult;
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
  void run(long steps);
  bool driverFaulted();

  HardwareSerial &uart_;
  TMC2209Stepper driver_;
  const char *error_ = nullptr;
};
