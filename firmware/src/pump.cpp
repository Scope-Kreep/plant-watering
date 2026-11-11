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
<<<<<<< Updated upstream
constexpr unsigned int kStepHighUs = 2;
constexpr unsigned int kStepLowUs = 300;
=======

// STEP-pulsen trenger bare noen hundre ns. Farten styres av perioden.
constexpr unsigned int kStepHighUs = 4;
constexpr unsigned int kStartPeriodUs = 1200; // rolig start og stopp
constexpr unsigned int kRunPeriodUs = 320;    // full fart, ca. 1 omdreining/s
constexpr long kRampSteps = 800;

// Tid fra driveren slås på til første steg, så StealthChop kan kalibrere i ro.
constexpr unsigned long kEnableSettleMs = 150;

// Les last fra driveren hvert N-te steg. Hver lesing gir ca. 1 ms pause
// i stegene. Sett til 0 for å slå av når feilsøkingen er ferdig.
constexpr long kDiagEvery = 800;
>>>>>>> Stashed changes
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

bool Pump::pulse() {
  error_ = nullptr;
  reconfigured_ = false;

  if (driver_.test_connection() != 0) {
    error_ = "driver_no_uart";
    return false;
  }

  if (driver_.reset()) {
    // Driveren har mistet strømmen (VM) og glemt innstillingene.
    configure();
    reconfigured_ = true;
  } else {
    // Nullstill gamle feil, så denne kjøringen rapporterer ferske verdier.
    driver_.GSTAT(0b110);
  }

  digitalWrite(kEnPin, LOW);
  delay(kEnableSettleMs);
  run(kPulseSteps);

  const bool faulted = driverFaulted();
  digitalWrite(kEnPin, HIGH);

  return !faulted;
}

Pump::Diag Pump::diagnostics() {
  Diag d;
  d.version = driver_.version();
  d.microsteps = driver_.microsteps();
  d.gstat = driver_.GSTAT();
  d.drvStatus = driver_.DRV_STATUS();
  d.reconfigured = reconfigured_;
  d.samples = samples_;
  d.sgMin = sgMin_;
  d.sgMax = sgMax_;
  d.csMin = csMin_;
  d.csMax = csMax_;
  return d;
}

void Pump::configure() {
  driver_.begin();
  driver_.toff(5);
  driver_.rms_current(kCurrentMa);
  driver_.microsteps(kMicrosteps);
  driver_.pwm_autoscale(true);
  driver_.en_spreadCycle(false);
  driver_.TCOOLTHRS(0xFFFFF); // slår på StallGuard-målingen (SG_RESULT)
  driver_.GSTAT(0b111);
}

void Pump::run(long steps) {
  resetLoadStats();
  digitalWrite(kDirPin, kForward ? HIGH : LOW);

  for (long i = 0; i < steps; ++i) {
    // Lineær rampe: treg ved start og slutt, full fart i midten.
    const long fromEdge = min(i, steps - 1 - i);
    unsigned int period = kRunPeriodUs;
    if (fromEdge < kRampSteps) {
      period = kStartPeriodUs -
               long(kStartPeriodUs - kRunPeriodUs) * fromEdge / kRampSteps;
    }

    digitalWrite(kStepPin, HIGH);
    delayMicroseconds(kStepHighUs);
    digitalWrite(kStepPin, LOW);
    delayMicroseconds(period - kStepHighUs);

    if (kDiagEvery > 0 && i > 0 && i % kDiagEvery == 0) {
      sampleLoad();
    }
  }
}

void Pump::resetLoadStats() {
  samples_ = 0;
  sgMin_ = 0xFFFF;
  sgMax_ = 0;
  csMin_ = 31;
  csMax_ = 0;
}

void Pump::sampleLoad() {
  const uint16_t sg = driver_.SG_RESULT();
  const uint8_t cs = (driver_.DRV_STATUS() >> 16) & 0x1F;

  if (sg < sgMin_) sgMin_ = sg;
  if (sg > sgMax_) sgMax_ = sg;
  if (cs < csMin_) csMin_ = cs;
  if (cs > csMax_) csMax_ = cs;
  if (samples_ < 255) ++samples_;
}

bool Pump::driverFaulted() {
  if (driver_.drv_err() || driver_.ot() || driver_.s2ga() || driver_.s2gb() ||
      driver_.s2vsa() || driver_.s2vsb()) {
    error_ = "driver_error";
    return true;
  }

  return false;
}
