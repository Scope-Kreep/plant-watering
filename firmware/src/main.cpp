#include <Arduino.h>
#include <TMCStepper.h>

// Pro Micro (ATmega32U4) -> TMC2209
//   TX0 (D1) --[1k]--+-- PDN_UART
//   RX1 (D0) --------+
//   VCC -> VIO, GND -> GND, MS1 + MS2 -> GND (UART address 0)
//   NB: the TMC2209 only answers on UART when motor power (VM) is on.

const int stepPin = 2; // blue wire, TMC2209 STEP
const int dirPin = 3;  // green wire, TMC2209 DIR
const int enPin = 4;   // TMC2209 EN, active low (LOW = motor on)

#define R_SENSE 0.11f    // ohm, typical for BTT/FYSETC modules, check yours
#define DRIVER_ADDR 0b00 // MS1 = GND, MS2 = GND
#define MOTOR_CURRENT 600 // mA RMS, keep below the motor's rated current
#define MICROSTEPS 16

const long fullStepsPerRev = 200; // 1.8 degree motor
const long stepsPerRev = fullStepsPerRev * MICROSTEPS;
const int pulseLength = 2;        // us, the TMC2209 needs at least ~0.1
const int stepDuration = 300;     // us between steps, lower = faster

// Serial1 is the 32U4 hardware UART on TX0/RX1. Serial stays free for USB.
TMC2209Stepper driver(&Serial1, R_SENSE, DRIVER_ADDR);

void moveSteps(long steps, bool forward) {
  digitalWrite(dirPin, forward ? HIGH : LOW);
  for (long i = 0; i < steps; i++) {
    digitalWrite(stepPin, HIGH);
    delayMicroseconds(pulseLength);
    digitalWrite(stepPin, LOW);
    delayMicroseconds(stepDuration);
  }
}

void setup() {
  pinMode(stepPin, OUTPUT);
  pinMode(dirPin, OUTPUT);
  pinMode(enPin, OUTPUT);
  digitalWrite(enPin, HIGH); // keep the motor off during configuration

  Serial.begin(115200);
  Serial1.begin(115200);

  // Give the USB serial port a moment so the first messages are not lost
  unsigned long start = millis();
  while (!Serial && millis() - start < 2000) {
  }

  driver.begin(); // UART control: pdn_disable + mstep_reg_select
  driver.toff(5); // enables the power stage
  driver.rms_current(MOTOR_CURRENT);
  driver.microsteps(MICROSTEPS);
  driver.pwm_autoscale(true);
  driver.en_spreadCycle(false); // StealthChop, quiet

  uint8_t result = driver.test_connection();
  if (result == 0) {
    Serial.print("TMC2209 OK, version 0x");
    Serial.println(driver.version(), HEX); // expected 0x21
  } else {
    // 1 = no reply (check VM, wiring, the 1k resistor)
    // 2 = reply is all zeros (PDN_UART is shorted to GND)
    Serial.print("TMC2209 UART ERROR, code ");
    Serial.println(result);
  }

  digitalWrite(enPin, LOW); // motor on
}

void loop() {
  moveSteps(stepsPerRev, true); // one revolution forward
  delay(1000);
  moveSteps(stepsPerRev, false);
  delay(1000);
}
