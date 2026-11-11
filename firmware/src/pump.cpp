#include "pump.h"
#include "Arduino.h"

namespace {
constexpr uint8_t kStepPin = 2;
constexpr uint8_t kDirPin = 3;
constexpr uint8_t kEnPin = 4;

constexpr float kRSense = 0.11f;
constexpr uint8_t kDriverAddr = 0b00;
constexpr uint16_t kCurrentMa = 600;
constexpr uint8_t kMicrosteps = 16;
constexpr unsigned long kUartBaud = 115200;

constexpr long kFullStepsPerRev = 200;
constexpr long kStepsPerRev = kFullStepsPerRev * kMicrosteps;

constexpr long kPulseSteps = kStepsPerRev;

constexpr bool kForward = true;
constexpr unsigned int kStepHighUs = 160;
constexpr unsigned int kStepLowUs = 160;
} // namespace

Pump::Pump(HardwareSerial &uart)
    : uart_(uart), driver_(&uart, kRSense, kDriverAddr) {}

bool Pump::begin() {
  error_ = nullptr;

  pinMode(kStepPin, OUTPUT);
  pinMode(kDirPin, OUTPUT);
  pinMode(kEnPin, OUTPUT);
  digitalWrite(kEnPin, HIGH);

  uart_.begin(kUartBaud);
  configure();

  if (driver_.test_connection() != 0) {
    error_ = "driver_no_uart";
    return false;
  }

  return true;
}

Pump::Reading Pump::pulse() {
  error_ = nullptr;

  Pump::Reading result{};

  if (driver_.test_connection() != 0) {
    error_ = "driver_no_uart";
    result.success = false;
    return result;
  }

  if (driver_.reset()) {
    configure();
  }

  digitalWrite(kEnPin, LOW);
  run(kPulseSteps);

  result.actualCurrentScale = driver_.cs_actual();
  result.stallGuardResult = driver_.SG_RESULT();
  result.pwmScale = driver_.pwm_scale_sum();
  result.microstepCounter = driver_.MSCNT();
  result.stepTime = driver_.TSTEP();
  result.success = !driverFaulted();

  digitalWrite(kEnPin, HIGH);

  return result;
}

void Pump::configure() {
  driver_.begin();
  driver_.toff(5);
  driver_.rms_current(kCurrentMa);
  driver_.microsteps(kMicrosteps);
  driver_.pwm_autoscale(true);
  driver_.en_spreadCycle(false);
  driver_.GSTAT(0b111);
}

void Pump::run(long steps) {
  digitalWrite(kDirPin, kForward ? HIGH : LOW);
  for (long i = 0; i < steps; ++i) {
    digitalWrite(kStepPin, HIGH);
    delayMicroseconds(kStepHighUs);
    digitalWrite(kStepPin, LOW);
    delayMicroseconds(kStepLowUs);
  }
}

bool Pump::driverFaulted() {
  if (driver_.drv_err() || driver_.ot() || driver_.s2ga() || driver_.s2gb() ||
      driver_.s2vsa() || driver_.s2vsb()) {
    error_ = "driver_error";
    return true;
  }

  return false;
}
