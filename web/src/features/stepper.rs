use jiff::Timestamp;

#[derive(Debug, toasty::Model)]
pub struct StepperMeasurement {
    #[key]
    #[auto]
    pub id: u64,

    pub current_scale: u16,
    pub pwm_scale: u8,
    pub microstep_counter: u16,
    pub step_time: u32,
    pub sg_samples: Vec<u16>,
    #[default(jiff::Timestamp::now())]
    pub timestamp: Timestamp,
}
