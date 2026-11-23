#include "pump.h"
#include <Arduino.h>
#include <util/atomic.h>

namespace {
constexpr uint8_t kStepPin = 2;
constexpr uint8_t kDirPin = 3;
constexpr uint8_t kEnPin = 4;
constexpr uint8_t kDiagPin = 7; // INT6 på ATmega32U4

constexpr float kRSense = 0.11f;
constexpr uint8_t kDriverAddr = 0b00;
constexpr uint16_t kCurrentMa = 1000;
constexpr uint8_t kMicrosteps = 16;
constexpr unsigned long kUartBaud = 115200;

constexpr long kFullStepsPerRev = 200;
constexpr long kStepsPerRev = kFullStepsPerRev * kMicrosteps;
constexpr long kPulseSteps = kStepsPerRev * 5;

constexpr bool kForward = true;
constexpr unsigned int kStartPeriodUs = 2000;
constexpr unsigned int kRunPeriodUs = 605;
constexpr long kRampSteps = 1000;

// TSTEP = tid mellom 1/256-mikrosteg i driverklokka (12 MHz)
constexpr uint32_t kRunTstep = uint32_t(kRunPeriodUs) * 12 * kMicrosteps / 256;
// Stalldeteksjon bare aktiv når farten er over ca. 2/3 av full fart
constexpr uint32_t kCoolThrs = kRunTstep * 3 / 2;
// 0 = stalldeteksjon av. Settes etter kalibrering (se under)
constexpr uint8_t kSgThreshold = 0;

volatile long gStepsTotal = 0;
volatile long gStepIndex = 0;
volatile long gRampSteps = 0;
volatile bool gRunning = false;
volatile bool gStalled = false;

uint16_t periodFor(long i, long total, long ramp) {
  const long fromEdge = min(i, total - 1 - i);
  if (ramp > 0 && fromEdge < ramp) {
    return kStartPeriodUs -
           long(kStartPeriodUs - kRunPeriodUs) * fromEdge / ramp;
  }
  return kRunPeriodUs;
}

void startTimer(uint16_t periodUs) {
  noInterrupts();
  TCCR1A = 0;
  TCCR1B = 0;
  TCNT1 = 0;
  OCR1A = periodUs * 2;              // 16 MHz / 8 = 0.5 µs per tick
  TCCR1B = _BV(WGM12) | _BV(CS11);   // CTC-modus, prescaler 8
  TIMSK1 |= _BV(OCIE1A);
  interrupts();
}

void stopTimer() {
  TIMSK1 &= ~_BV(OCIE1A);
  TCCR1B = 0;
  gRunning = false;
}

void onDiag() {
  gStalled = true;
  stopTimer();
}
} // namespace

ISR(TIMER1_COMPA_vect) {
  if (gStepIndex >= gStepsTotal) {
    stopTimer();
    return;
  }
  digitalWrite(kStepPin, HIGH);
  digitalWrite(kStepPin, LOW);
  ++gStepIndex;
  if (gStepIndex < gStepsTotal) {
    OCR1A = periodFor(gStepIndex, gStepsTotal, gRampSteps) * 2;
  }
}

Pump::Pump(HardwareSerial &uart)
    : uart_(uart), driver_(&uart, kRSense, kDriverAddr) {}

bool Pump::begin() {
  error_ = nullptr;

  pinMode(kStepPin, OUTPUT);
  pinMode(kDirPin, OUTPUT);
  pinMode(kEnPin, OUTPUT);
  pinMode(kDiagPin, INPUT);
  digitalWrite(kEnPin, HIGH);

  uart_.begin(kUartBaud);
  configure();

  attachInterrupt(digitalPinToInterrupt(kDiagPin), onDiag, RISING);

  return testConnection();
}

void Pump::configure() {
  driver_.begin();
  driver_.I_scale_analog(false);
  driver_.internal_Rsense(false);
  driver_.toff(4);
  driver_.blank_time(24);
  driver_.rms_current(kCurrentMa);
  driver_.microsteps(kMicrosteps);
  driver_.en_spreadCycle(false);  // StealthChop: kreves for StallGuard4
  driver_.pwm_autoscale(true);
  driver_.pwm_autograd(true);
  driver_.TPWMTHRS(0);            // alltid StealthChop
  driver_.TCOOLTHRS(kCoolThrs);
  driver_.SGTHRS(kSgThreshold);
  driver_.GSTAT(0b111);
}

void Pump::pulse(Reading &result) {
  error_ = nullptr;
  result = {};

  if (!testConnection()) {
    result.success = false;
    return;
  }
  if (driver_.reset()) {
    configure();
  }

  digitalWrite(kDirPin, kForward ? HIGH : LOW);
  digitalWrite(kEnPin, LOW);
  delay(150); // StealthChop-autotuning trenger stillstand etter enable

  const long ramp = min(kRampSteps, kPulseSteps / 2);
  const long cruise = kPulseSteps - 2 * ramp;
  const long sampleEvery = max(1L, cruise / (kMaxLoadSamples + 1));
  long nextSample = ramp + sampleEvery;

  gStepsTotal = kPulseSteps;
  gStepIndex = 0;
  gRampSteps = ramp;
  gStalled = false;
  gRunning = true;
  startTimer(kStartPeriodUs);

  // Timeren stepper i bakgrunnen, så UART-lesing her stopper ikke motoren
  while (gRunning) {
    long i;
    ATOMIC_BLOCK(ATOMIC_RESTORESTATE) { i = gStepIndex; }

    if (i >= nextSample && i < kPulseSteps - ramp &&
        result.sgSampleCount < kMaxLoadSamples) {
      result.sgSamples[result.sgSampleCount++] = driver_.SG_RESULT();
      if (result.sgSampleCount == 1) {
        result.stepTime = driver_.TSTEP();        // målt under kjøring
        result.pwmScale = driver_.pwm_scale_sum();
      }
      nextSample += sampleEvery;
    }
  }

  result.stalled = gStalled;
  result.currentScale = driver_.cs_actual();
  result.microstepCounter = driver_.MSCNT();
  result.success = !gStalled && !driverFaulted();
  if (gStalled && error_ == nullptr) {
    error_ = "stepper_stalled";
  }

  digitalWrite(kEnPin, HIGH);
}
