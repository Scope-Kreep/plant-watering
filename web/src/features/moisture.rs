use jiff::Timestamp;

#[derive(Debug, toasty::Model)]
pub struct Moisture {
    #[key]
    #[auto]
    pub id: u64,

    pub raw: u16,
    pub spread: u16,
    pub percent: f32,
    #[default(jiff::Timestamp::now())]
    pub timestamp: Timestamp,
}
