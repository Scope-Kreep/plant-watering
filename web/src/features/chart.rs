use jiff::Timestamp;
use std::fmt::Write;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::view::{View, component, view};

use crate::components::card::{card, card_content, card_description, card_header, card_title};
use crate::features::{age_secs, human};

use super::Window;

const BUCKETS: usize = 120;
const GAP_BUCKETS: usize = 12;

pub struct Series {
    pub path: String,
    pub top: String,
    pub bottom: String,
    pub stats: String,
}

pub fn series(
    points: &[(Timestamp, f32)],
    w: &Window,
    fixed: Option<(f32, f32)>,
    unit: &str,
    dec: usize,
    trend: bool,
) -> Series {
    if points.is_empty() {
        return Series {
            path: String::new(),
            top: String::new(),
            bottom: String::new(),
            stats: "No data in this range".to_string(),
        };
    }

    let vals: Vec<f32> = points.iter().map(|p| p.1).collect();
    let min = vals.iter().copied().fold(f32::INFINITY, f32::min);
    let max = vals.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let avg = vals.iter().sum::<f32>() / vals.len() as f32;
    let (lo, hi) = fixed.unwrap_or_else(|| widen(min, max));

    let mut stats = format!(
        "{} readings · min {min:.dec$}{unit} · avg {avg:.dec$}{unit} · max {max:.dec$}{unit}",
        points.len()
    );
    if trend {
        if let Some(s) = slope_per_hour(points) {
            let _ = write!(stats, " · trend {s:+.1}{unit}/h");
        }
    }

    if let Some(gap) = longest_gap_secs(points) {
        let _ = write!(stats, " · longest gap {}", human(gap));
    }

    Series {
        path: path(&bucketize(points, w), lo, hi),
        top: format!("{hi:.dec$}{unit}"),
        bottom: format!("{lo:.dec$}{unit}"),
        stats,
    }
}

fn bucketize(points: &[(Timestamp, f32)], w: &Window) -> Vec<Option<f32>> {
    let mut sum = vec![0.0f32; BUCKETS];
    let mut count = vec![0u32; BUCKETS];
    for (ts, v) in points {
        let t = (ts.as_second() - w.start.as_second()) as f64 / w.span_secs();
        if !(0.0..=1.0).contains(&t) {
            continue;
        }
        let i = ((t * BUCKETS as f64) as usize).min(BUCKETS - 1);
        sum[i] += v;
        count[i] += 1;
    }
    sum.iter()
        .zip(&count)
        .map(|(s, c)| (*c > 0).then(|| s / *c as f32))
        .collect()
}

pub fn path(buckets: &[Option<f32>], lo: f32, hi: f32) -> String {
    let n = buckets.len();
    let span = (hi - lo).max(f32::EPSILON);
    let mut d = String::new();
    let mut last: Option<usize> = None;
    for (i, b) in buckets.iter().enumerate() {
        let Some(v) = b else { continue };
        let x = (i as f32 + 0.5) / n as f32 * 100.0;
        let y = 38.0 - ((v - lo) / span).clamp(0.0, 1.0) * 36.0;
        let joined = last.is_some_and(|l| i - l <= GAP_BUCKETS);
        let _ = write!(d, "{}{x:.2} {y:.2} ", if joined { "L" } else { "M" });
        last = Some(i)
    }
    d
}

pub fn widen(min: f32, max: f32) -> (f32, f32) {
    if max - min < 1e-6 {
        (min - 1.0, max + 1.0)
    } else {
        (min, max)
    }
}

fn slope_per_hour(points: &[(Timestamp, f32)]) -> Option<f32> {
    if points.len() < 2 {
        return None;
    }

    let t0 = points[0].0.as_second();
    let xs: Vec<f64> = points
        .iter()
        .map(|(t, _)| (t.as_second() - t0) as f64 / 3600.0)
        .collect();
    let ys: Vec<f64> = points.iter().map(|(_, v)| *v as f64).collect();
    let n = xs.len() as f64;
    let (mx, my) = (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n);
    let var: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
    if var < 1e-9 {
        return None;
    }
    let cov: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    Some((cov / var) as f32)
}

fn longest_gap_secs(points: &[(Timestamp, f32)]) -> Option<i64> {
    points
        .windows(2)
        .map(|p| p[1].0.as_second() - p[0].0.as_second())
        .max()
}

#[component]
pub async fn line_chart(
    cx: &Cx,
    title: &str,
    description: &str,
    series: &Series,
    window: &Window,
) -> Result<impl View> {
    Ok(view! {
        card(
            card_header(
                card_title((title))
                card_description((description))
            )
            card_content(
                if series.path.is_empty() {
                    <div
                        class="flex h-32 items-center justify-center text-sm text-muted-foreground"
                    >
                        "No data in this range"
                    </div>
                } else {
                    <div class="relative">
                        <svg
                            viewBox="0 0 100 40"
                            preserveAspectRatio="none"
                            class="h-32 w-full text-primary"
                        >
                            <g class="text-foreground">
                                <line
                                    x1="0"
                                    x2="100"
                                    y1="2"
                                    y2="2"
                                    stroke="currentColor"
                                    stroke-opacity="0.12"
                                    vector-effect="non-scaling-stroke"
                                ></line>
                                <line
                                    x1="0"
                                    x2="100"
                                    y1="20"
                                    y2="20"
                                    stroke="currentColor"
                                    stroke-opacity="0.12"
                                    vector-effect="non-scaling-stroke"
                                ></line>
                                <line
                                    x1="0"
                                    x2="100"
                                    y1="38"
                                    y2="38"
                                    stroke="currentColor"
                                    stroke-opacity="0.12"
                                    vector-effect="non-scaling-stroke"
                                ></line>
                            </g>
                            <path
                                d=(&series.path)
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linejoin="round"
                                vector-effect="non-scaling-stroke"
                            ></path>
                        </svg>
                        <span
                            class="absolute right-0 top-0 text-xs text-muted-foreground"
                        >
                            (&series.top)
                        </span>
                        <span
                            class="absolute bottom-0 right-0 text-xs text-muted-foreground"
                        >
                            (&series.bottom)
                        </span>
                    </div>
                }
                <div class="mt-1 flex justify-between text-xs text-muted-foreground">
                    <span>(window.start_label())</span>
                    <span>(window.end_label())</span>
                </div>
                <p class="mt-3 text-sm text-muted-foreground">(&series.stats)</p>
            )
        )
    })
}

#[component]
pub async fn stat_tile(
    cx: &Cx,
    title: &str,
    value: &str,
    detail: &str,
    updated_at: Option<Timestamp>,
    warning: &str,
) -> Result<impl View> {
    Ok(view! {
        card(
            card_header(
                card_title((title))
                card_description(updated_ago(at: updated_at))
            )
            card_content(
                <div class="text-4xl font-semibold tabular-nums">(value)</div>
                <div class="mt-1 text-sm text-muted-foreground">(detail)</div>
                if !warning.is_empty() {
                    <p class="mt-2 text-sm font-medium text-red-500">(warning)</p>
                }
            )
        )
    })
}

#[component]
pub async fn updated_ago(cx: &Cx, at: Option<Timestamp>) -> Result<impl View> {
    let now = Timestamp::now();
    let text = match at {
        Some(s) => format!("updated {} ago", human(age_secs(s, now))),
        None => "never updated".to_string(),
    };
    Ok(view! {
        <span data-updated-at=(at.map(|s| s.to_string()).unwrap_or_default())>
            (text)
        </span>
    })
}
