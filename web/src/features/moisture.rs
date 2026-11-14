use jiff::Timestamp;
use topcoat::{
    Result,
    context::Cx,
    runtime::{connected, shard},
    view::{View, component, emit, live},
};

use crate::{
    db::db,
    features::{
        Kind, Range, STALE_SECS, age_secs,
        chart::{line_chart, series, stat_tile},
        human, subscribe, wait_for,
    },
};

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

#[component]
pub async fn moisture_tile(cx: &Cx) -> Result<impl View> {
    Ok(live! {
        let mut rx = subscribe(cx);
        loop {
            let latest = Moisture::all()
                .order_by(Moisture::fields().id().desc())
                .first()
                .exec(&mut db(cx))
                .await?;
            let now = Timestamp::now();

            let (value, detail, updated, warning) = match latest {
                Some(m) => {
                    let age = age_secs(m.timestamp, now);
                    (
                        format!("{:.0}%", m.percent),
                        format!("raw {} · spread {}", m.raw, m.spread),
                        format!("updated {} ago", human(age)),
                        if age > STALE_SECS {
                            format!("No new reading for {}", human(age))
                        } else {
                            String::new()
                        },
                    )
                }
                None => (
                    "—".to_string(),
                    "No readings yet".to_string(),
                    "never updated".to_string(),
                    String::new(),
                ),
            };

            let token = emit! {
                stat_tile(
                    title: "Soil moisture",
                    value: &value,
                    detail: &detail,
                    updated: &updated,
                    warning: &warning
                )
            }?;

            if !connected(cx) {
                break Ok(token);
            }
            wait_for(&mut rx, Kind::Moisture).await;
        }
    })
}

#[shard]
pub async fn moisture_charts(cx: &Cx, range: String) -> Result<impl View> {
    let range = Range::from_slug(&range);
    Ok(live! {
        let mut rx = subscribe(cx);
        loop {
            let w = range.window();
            // GAP: `.ge(Timestamp)` on a jiff field is assumed, I only saw `.eq` and `.like`.
            let rows = Moisture::filter(Moisture::fields().timestamp().ge(w.start))
                .order_by(Moisture::fields().id().asc())
                .exec(&mut db(cx))
                .await?;

            let pct: Vec<_> = rows.iter().map(|r| (r.timestamp, r.percent)).collect();
            let spread: Vec<_> = rows
                .iter()
                .map(|r| (r.timestamp, f32::from(r.spread)))
                .collect();
            let pct = series(&pct, &w, Some((0.0, 100.0)), "%", 0, true);
            let spread = series(&spread, &w, None, "", 0, false);

            let token = emit! {
                <div class="grid gap-4 md:grid-cols-2">
                    line_chart(
                        title: "Soil moisture",
                        description: "Average per time bucket",
                        series: &pct,
                        window: &w
                    )
                    line_chart(
                        title: "Sensor spread",
                        description: "spread, averaged per time bucket",
                        series: &spread,
                        window: &w
                    )
                </div>
            }?;

            if !connected(cx) {
                break Ok(token);
            }
            // Wake on new data, or every 30 s so the window keeps sliding.
            wait_for(&mut rx, Kind::Moisture).await;
        }
    })
}
