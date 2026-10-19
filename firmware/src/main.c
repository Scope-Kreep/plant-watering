#include "Arduino.h"

const int stepPin = 2;         // blue wire, A4988 STEP
const int dirPin = 3;          // green wire, A4988 DIR
const int pulseLength = 5;     // us, the A4988 needs at least 1
const int stepDuration = 5000; // us between steps, lower = faster

void moveSteps(int steps, bool forward) {
  digitalWrite(dirPin, forward ? HIGH : LOW);
  for (int i = 0; i < steps; i++) {
    digitalWrite(stepPin, HIGH);
    delayMicroseconds(pulseLength);
    digitalWrite(stepPin, LOW);
    delayMicroseconds(stepDuration);
  }
}

void setup() {
  pinMode(stepPin, OUTPUT);
  pinMode(dirPin, OUTPUT);
}

void loop() {
  moveSteps(200, true);
  delay(1000);
  moveSteps(200, false);
  delay(1000);
}
