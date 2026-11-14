use jiff::Timestamp;

#[derive(Debug, toasty::Model)]
pub struct State {
    #[key]
    #[auto]
    pub id: u64,

    pub name: String,
    pub reason: Option<String>,
    #[default(jiff::Timestamp::now())]
    pub timestamp: Timestamp,
}
