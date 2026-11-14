use jiff::{SignedDuration, Timestamp, tz::TimeZone};
use tokio::sync::broadcast::{self, error::RecvError};
use topcoat::context::{Cx, context};

pub mod chart;
pub mod moisture;
pub mod state;
pub mod stepper;

pub const STALE_SECS: i64 = 5 * 60;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Kind {
    Moisture,
    State,
    Stepper,
}

#[derive(Clone)]
pub struct PlantEvents(broadcast::Sender<Kind>);

impl Default for PlantEvents {
    fn default() -> Self {
        Self(broadcast::channel(64).0)
    }
}

impl PlantEvents {
    pub fn notify(&self, kind: Kind) {
        let _ = self.0.send(kind);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Kind> {
        self.0.subscribe()
    }
}

pub fn subscribe(cx: &Cx) -> broadcast::Receiver<Kind> {
    context::<PlantEvents>(cx).subscribe()
}

pub async fn wait_for(rx: &mut broadcast::Receiver<Kind>, want: Kind) {
    loop {
        match rx.recv().await {
            Ok(kind) if kind == want => return,
            Ok(_) => {}
            Err(RecvError::Lagged(_)) | Err(RecvError::Closed) => return,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Range {
    Hour,
    #[default]
    SixHours,
    Day,
    Week,
}

impl Range {
    pub const ALL: [Range; 4] = [Range::Hour, Range::SixHours, Range::Day, Range::Week];

    pub fn slug(self) -> &'static str {
        match self {
            Range::Hour => "1h",
            Range::SixHours => "6h",
            Range::Day => "24h",
            Range::Week => "7d",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Range::Hour => "Last hour",
            Range::SixHours => "Last 6 hours",
            Range::Day => "Last 24 hours",
            Range::Week => "Last 7 days",
        }
    }

    pub fn from_slug(slug: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|r| r.slug() == slug)
            .unwrap_or_default()
    }

    fn duration(self) -> SignedDuration {
        match self {
            Range::Hour => SignedDuration::from_hours(1),
            Range::SixHours => SignedDuration::from_hours(6),
            Range::Day => SignedDuration::from_hours(24),
            Range::Week => SignedDuration::from_hours(24 * 7),
        }
    }

    pub fn window(self) -> Window {
        let end = Timestamp::now();
        let start = end
            .checked_sub(self.duration())
            .unwrap_or(Timestamp::UNIX_EPOCH);
        Window {
            start,
            end,
            range: self,
        }
    }
}

pub struct Window {
    pub start: Timestamp,
    pub end: Timestamp,
    pub range: Range,
}

impl Window {
    pub fn span_secs(&self) -> f64 {
        ((self.end.as_second() - self.start.as_second()) as f64).max(1.0)
    }

    pub fn frac(&self, ts: Timestamp) -> f64 {
        ((ts.as_second() - self.start.as_second()) as f64 / self.span_secs()).clamp(0.0, 1.0)
    }

    pub fn start_label(&self) -> String {
        axis(self.start, self.range)
    }

    pub fn end_label(&self) -> String {
        axis(self.end, self.range)
    }
}

pub fn clock(ts: Timestamp) -> String {
    ts.to_zoned(TimeZone::system())
        .strftime("%H:%M:%S")
        .to_string()
}

pub fn axis(ts: Timestamp, range: Range) -> String {
    let fmt = match range {
        Range::Hour | Range::SixHours => "%H:%M",
        Range::Day | Range::Week => "%a %H:%M",
    };
    ts.to_zoned(TimeZone::system()).strftime(fmt).to_string()
}

pub fn human(secs: i64) -> String {
    let s = secs.max(0);
    match s {
        0..=59 => format!("{s}s"),
        60..=3599 => format!("{}m", s / 60),
        3600..=86399 => format!("{}h {}m", s / 3600, s % 3600 / 60),
        _ => format!("{}d {}h", s / 86400, s % 86400 / 3600),
    }
}

pub fn age_secs(ts: Timestamp, now: Timestamp) -> i64 {
    (now.as_second() - ts.as_second()).max(0)
}
