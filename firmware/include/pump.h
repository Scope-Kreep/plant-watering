#pragma once

#include <Arduino.h>
#include <TMCStepper.h>

class Pump {
public:
  Pump(HardwareSerial &uart);

  bool begin();

  bool pulse();

  const char *error() const { return error_; }

private:
  void configure();
  void run(long steps);
  bool driverFaulted();

  HardwareSerial &uart_;
  TMC2209Stepper driver_;
  const char *error_ = nullptr;
};
