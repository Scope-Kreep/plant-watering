#include "Arduino.h"

const int stepPin = 3;
const int dirPin = 2;
const int pulseLength = 5;
const int stepDuration = 500;

void setup() {
  pinMode(stepPin, OUTPUT);
  pinMode(dirPin, OUTPUT);
}

void loop() {
  digitalWrite(dirPin, HIGH);

  for (int x = 0; x < 200; x++) {
    digitalWrite(stepPin, HIGH);
    delayMicroseconds(pulseLength);
    digitalWrite(stepPin, LOW);
    delayMicroseconds(stepDuration);
  }
  delay(1000);

  digitalWrite(dirPin, LOW);
  for (int x = 0; x < 400; x++) {
    digitalWrite(stepPin, HIGH);
    delayMicroseconds(pulseLength);
    digitalWrite(stepPin, LOW);
    delayMicroseconds(stepDuration);
  }
  delay(1000);
}
