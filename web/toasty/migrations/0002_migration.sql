CREATE TABLE "stepper_measurements" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "current_scale" INTEGER NOT NULL,
    "pwm_scale" INTEGER NOT NULL,
    "microstep_counter" INTEGER NOT NULL,
    "step_time" INTEGER NOT NULL,
    "sg_samples" TEXT NOT NULL,
    "timestamp" TEXT NOT NULL
);
