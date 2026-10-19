#include "Arduino.h"

// Stepper som spinner kontinuerlig mot venstre – A4988
// STEP -> pin 2
// DIR  -> pin 3

const int STEP_PIN = 2;
const int DIR_PIN = 3;

const unsigned int STEG_PAUSE_US =
    1000; // tid mellom steg i µs – mindre tall = raskere

void setup() {
  pinMode(STEP_PIN, OUTPUT);
  pinMode(DIR_PIN, OUTPUT);
  digitalWrite(STEP_PIN, LOW);
  digitalWrite(DIR_PIN,
               LOW); // LOW = venstre (bytt til HIGH hvis den går feil vei)
  delayMicroseconds(5);
}

void loop() {
  digitalWrite(STEP_PIN, HIGH);
  delayMicroseconds(5); // A4988 trenger minst 1 µs høy puls
  digitalWrite(STEP_PIN, LOW);
  delayMicroseconds(STEG_PAUSE_US);
}
