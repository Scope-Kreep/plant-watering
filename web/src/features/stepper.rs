use jiff::Timestamp;

#[derive(Debug, toasty::Model)]
struct StepperMeasurement {
    #[key]
    #[auto]
    id: u64,

    current_scale: u16,
    pwm_scale: u8,
    microstep_counter: u16,
    step_time: u32,
    sg_samples: Vec<u16>,
    timestamp: Timestamp,
}
