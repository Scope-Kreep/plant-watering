use jiff::Timestamp;

#[derive(Debug, toasty::Model)]
struct Moisture {
    #[key]
    #[auto]
    id: u64,

    raw: u16,
    spread: u16,
    percent: f32,
    timestamp: Timestamp,
}
