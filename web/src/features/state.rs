use jiff::Timestamp;

#[derive(Debug, toasty::Model)]
struct State {
    #[key]
    #[auto]
    id: u64,

    name: String,
    reason: Option<String>,
    timestamp: Timestamp,
}
