#pragma once

#include <Arduino.h>
#include <TMCStepper.h>

class Pump {
public:
  // Øyeblikksbilde av driveren etter siste pulse (eller etter begin()).
  struct Diag {
    uint8_t version;     // 0x21 (33) = TMC2209 svarer på UART
    uint16_t microsteps; // skal være lik kMicrosteps
    uint8_t gstat;       // bit0 reset, bit1 drv_err, bit2 uv_cp
    uint32_t drvStatus;  // rå DRV_STATUS
    bool reconfigured;   // driveren hadde mistet innstillingene før siste pulse
    uint8_t samples;     // antall lastmålinger under siste pulse
    uint16_t sgMin, sgMax; // StallGuard: høy = lett last, nær 0 = nesten stopp
    uint8_t csMin, csMax;  // faktisk strømskala 0-31
  };

  explicit Pump(HardwareSerial &uart);

  bool begin();
  bool pulse();
  Diag diagnostics();
  const char *error() const { return error_; }

private:
  void configure();
  void run(long steps);
  bool driverFaulted();
  void resetLoadStats();
  void sampleLoad();

  HardwareSerial &uart_;
  TMC2209Stepper driver_;
  const char *error_ = nullptr;

  bool reconfigured_ = false;
  uint8_t samples_ = 0;
  uint16_t sgMin_ = 0;
  uint16_t sgMax_ = 0;
  uint8_t csMin_ = 0;
  uint8_t csMax_ = 0;
};
