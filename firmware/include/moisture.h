#pragma once

#include <Arduino.h>

class MoistureSensor {
public:
  struct Reading {
    uint16_t raw;
    uint16_t spread;
    float percent;
  };

  MoistureSensor(uint8_t pin);

  Reading read() const;

private:
  static float toPercent(uint16_t raw);

  uint8_t pin_;
};
