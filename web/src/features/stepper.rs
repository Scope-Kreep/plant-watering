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
        chart::{Series, line_chart, path, series, stat_tile, widen},
        human, subscribe, wait_for,
    },
};

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

fn mean(samples: &[u16]) -> Option<f32> {
    (!samples.is_empty())
        .then(|| samples.iter().map(|&v| f32::from(v)).sum::<f32>() / samples.len() as f32)
}

#[component]
pub async fn stepper_tile(cx: &Cx) -> Result<impl View> {
    Ok(live! {
        let mut rx = subscribe(cx);
        loop {
            let latest = StepperMeasurement::all()
                .order_by(StepperMeasurement::fields().id().desc())
                .first()
                .exec(&mut db(cx))
                .await?;
            let now = Timestamp::now();

            let (value, detail, updated_at, warning) = match latest {
                Some(m) => {
                    let age = age_secs(m.timestamp, now);
                    (
                        format!("{} / {}", m.current_scale, m.pwm_scale),
                        format!(
                            "current_scale / pwm_scale · microstep_counter {} · step_time {}",
                            m.microstep_counter,
                            m.step_time,
                        ),
                        Some(m.timestamp),
                        if age > STALE_SECS {
                            format!("No new measurement for {}", human(age))
                        } else {
                            String::new()
                        },
                    )
                }
                None => (
                    "—".to_string(),
                    "No measurements yet".to_string(),
                    None,
                    String::new(),
                ),
            };

            let token = emit! {
                stat_tile(
                    title: "Stepper",
                    value: &value,
                    detail: &detail,
                    updated_at: updated_at,
                    warning: &warning
                )
            }?;

            if !connected(cx) {
                break Ok(token);
            }
            wait_for(&mut rx, Kind::Stepper).await;
        }
    })
}

#[shard]
pub async fn stepper_charts(cx: &Cx, range: String) -> Result<impl View> {
    let range = Range::from_slug(&range);
    Ok(live! {
        let mut rx = subscribe(cx);
        loop {
            let w = range.window();
            let rows = StepperMeasurement::filter(
                StepperMeasurement::fields().timestamp().ge(w.start),
            ).order_by(StepperMeasurement::fields().id().asc()).exec(&mut db(cx)).await?;

            let sg_points: Vec<_> = rows
                .iter()
                .filter_map(|r| mean(&r.sg_samples).map(|m| (r.timestamp, m)))
                .collect();
            let step_points: Vec<_> = rows
                .iter()
                .map(|r| (r.timestamp, r.step_time as f32))
                .collect();
            let sg_mean = series(&sg_points, &w, None, "", 0, false);
            let step_time = series(&step_points, &w, None, "", 0, false);

            // The newest pulse is shown regardless of the selected range:
            // the samples cover the few seconds the motor was running,
            // not the dashboard time window.
            let latest = StepperMeasurement::all()
                .order_by(StepperMeasurement::fields().id().desc())
                .first()
                .exec(&mut db(cx))
                .await?;
            let (latest_sg, latest_end) = latest_pulse(latest.as_ref(), Timestamp::now());
            let (x_start, x_end) = (w.start_label(), w.end_label());

            let token = emit! {
                <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                    line_chart(
                        title: "sg_samples, mean per measurement",
                        description: "Averaged per time bucket",
                        series: &sg_mean,
                        x_start: x_start.as_str(),
                        x_end: x_end.as_str()
                    )
                    line_chart(
                        title: "step_time",
                        description: "Averaged per time bucket",
                        series: &step_time,
                        x_start: x_start.as_str(),
                        x_end: x_end.as_str()
                    )
                    line_chart(
                        title: "Latest sg_samples",
                        description: "Newest pump pulse, by sample index",
                        series: &latest_sg,
                        x_start: "sample 1",
                        x_end: latest_end.as_str()
                    )
                </div>
            }?;

            if !connected(cx) {
                break Ok(token);
            }
            wait_for(&mut rx, Kind::Stepper).await;
        }
    })
}

/// Builds the chart for one pump pulse, with sample index on the x-axis.
/// Returns the series and the label for the last sample.
fn latest_pulse(m: Option<&StepperMeasurement>, now: Timestamp) -> (Series, String) {
    let Some(m) = m.filter(|m| !m.sg_samples.is_empty()) else {
        return (
            Series {
                path: String::new(),
                top: String::new(),
                bottom: String::new(),
                stats: "No pump pulse with samples yet".to_string(),
            },
            String::new(),
        );
    };

    let vals: Vec<f32> = m.sg_samples.iter().map(|&v| f32::from(v)).collect();
    let min = vals.iter().copied().fold(f32::INFINITY, f32::min);
    let max = vals.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let avg = vals.iter().sum::<f32>() / vals.len() as f32;
    let (lo, hi) = widen(min, max);
    let buckets: Vec<Option<f32>> = vals.iter().map(|v| Some(*v)).collect();

    let series = Series {
        path: path(&buckets, lo, hi),
        top: format!("{hi:.0}"),
        bottom: format!("{lo:.0}"),
        stats: format!(
            "{} samples · min {min:.0} · avg {avg:.0} · max {max:.0} · pulse {} ago",
            vals.len(),
            human(age_secs(m.timestamp, now)),
        ),
    };
    (series, format!("sample {}", vals.len()))
}
