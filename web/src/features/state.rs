use jiff::Timestamp;
use topcoat::Result;
use topcoat::runtime::{connected, shard};
use topcoat::view::{View, emit, live};
use topcoat::{context::Cx, view::component};

use crate::components::card::{card, card_content, card_description, card_header, card_title};
use crate::components::table::{
    table, table_body, table_cell, table_head, table_header, table_row,
};
use crate::db::db;
use crate::features::chart::updated_ago;
use crate::features::{Kind, Range, human, subscribe, wait_for};

use super::{Window, clock};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StateKind {
    Boot,
    Idle,
    Measure,
    Fault,
    Other,
}

impl StateKind {
    const ALL: [StateKind; 5] = [
        StateKind::Boot,
        StateKind::Idle,
        StateKind::Measure,
        StateKind::Fault,
        StateKind::Other,
    ];

    fn from_name(name: &str) -> Self {
        match name.trim().to_ascii_lowercase().as_str() {
            "boot" => StateKind::Boot,
            "idle" => StateKind::Idle,
            "measure" => StateKind::Measure,
            "fault" => StateKind::Fault,
            _ => StateKind::Other,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            StateKind::Boot => "Boot",
            StateKind::Idle => "Idle",
            StateKind::Measure => "Measure",
            StateKind::Fault => "Fault",
            StateKind::Other => "Other",
        }
    }

    fn bg(&self) -> &'static str {
        match self {
            StateKind::Boot => "bg-slate-500",
            StateKind::Idle => "bg-emerald-500",
            StateKind::Measure => "bg-blue-500",
            StateKind::Fault => "bg-red-500",
            StateKind::Other => "bg-zinc-400",
        }
    }
}

struct Block {
    kind: StateKind,
    name: String,
    reason: Option<String>,
    from: Timestamp,
    to: Timestamp,
}

impl Block {
    fn secs(&self) -> i64 {
        (self.to.as_second() - self.from.as_second()).max(0)
    }

    fn style(&self, w: &Window) -> String {
        let left = w.frac(self.from) * 100.0;
        let width = (w.frac(self.to) * 100.0 - left).max(0.0);
        format!("left:{left:.3}%;width:{width:.3}%;min-width:2px")
    }

    fn title(&self) -> String {
        format!("{} · {} to {}", self.name, clock(self.from), clock(self.to))
    }
}

fn build_blocks(before: Option<&State>, rows: &[State], w: &Window) -> Vec<Block> {
    let mut starts: Vec<(&State, Timestamp)> = Vec::new();
    if let Some(b) = before {
        starts.push((b, w.start));
    }
    starts.extend(rows.iter().map(|r| (r, r.timestamp)));

    starts
        .iter()
        .enumerate()
        .map(|(i, (s, from))| {
            let to = starts.get(i + 1).map_or(w.end, |(_, t)| *t);
            Block {
                kind: StateKind::from_name(&s.name),
                name: s.name.clone(),
                reason: s.reason.clone(),
                from: (*from).min(w.end),
                to: to.min(w.end),
            }
        })
        .collect()
}

struct Share {
    label: &'static str,
    bg: &'static str,
    secs: i64,
    pct: f64,
}

fn shares(blocks: &[Block], w: &Window) -> Vec<Share> {
    StateKind::ALL
        .into_iter()
        .filter_map(|kind| {
            let secs: i64 = blocks
                .iter()
                .filter(|b| b.kind == kind)
                .map(Block::secs)
                .sum();
            (secs > 0 || kind != StateKind::Other).then(|| Share {
                label: kind.label(),
                bg: kind.bg(),
                secs,
                pct: secs as f64 / w.span_secs() * 100.0,
            })
        })
        .collect()
}

struct LogRow {
    time: String,
    name: String,
    reason: String,
    bg: &'static str,
}

#[component]
pub async fn state_tile(cx: &Cx) -> Result<impl View> {
    Ok(live! {
        let mut rx = subscribe(cx);
        loop {
            let latest = State::all()
                .order_by(State::fields().id().desc())
                .first()
                .exec(&mut db(cx))
                .await?;

            let (name, bg, updated_at, reason) = match latest {
                Some(s) => (
                    s.name.clone(),
                    StateKind::from_name(&s.name).bg(),
                    Some(s.timestamp),
                    s.reason.clone().unwrap_or_default(),
                ),
                None => ("—".to_string(), StateKind::Other.bg(), None, String::new()),
            };

            let token = emit! {
                card(
                    card_header(
                        card_title("State")
                        card_description(updated_ago(at: updated_at))
                    )
                    card_content(
                        <span
                            class=(format!(
                                "inline-flex rounded-md px-3 py-1 text-2xl font-semibold text-white {bg}",
                            ))
                        >
                            (name)
                        </span>
                        if !reason.is_empty() {
                            <div class="mt-2 text-sm text-muted-foreground">
                                (reason)
                            </div>
                        }
                    )
                )
            }?;

            if !connected(cx) {
                break Ok(token);
            }
            wait_for(&mut rx, Kind::State).await;
        }
    })
}

#[shard]
pub async fn state_section(cx: &Cx, range: String) -> Result<impl View> {
    let range = Range::from_slug(&range);
    Ok(live! {
        let mut rx = subscribe(cx);
        loop {
            let w = range.window();
            let mut db = db(cx);

            let rows = State::filter(State::fields().timestamp().ge(w.start))
                .order_by(State::fields().id().asc())
                .exec(&mut db)
                .await?;
            let before = State::filter(State::fields().timestamp().lt(w.start))
                .order_by(State::fields().id().desc())
                .first()
                .exec(&mut db)
                .await?;

            let blocks = build_blocks(before.as_ref(), &rows, &w);
            let shares = shares(&blocks, &w);

            let faults: Vec<&Block> = blocks
                .iter()
                .filter(|b| b.kind == StateKind::Fault)
                .collect();
            let fault_line = match faults.last() {
                Some(b) => format!(
                    "{} in range · last at {}{}",
                    faults.len(),
                    clock(b.from),
                    b.reason.as_ref().map(|r| format!(" — {r}")).unwrap_or_default(),
                ),
                None => "None in range".to_string(),
            };

            let log: Vec<LogRow> = rows
                .iter()
                .rev()
                .take(10)
                .map(
                    |s| LogRow {
                            time: clock(s.timestamp),
                            name: s.name.clone(),
                            reason: s
                                .reason
                                .clone()
                                .unwrap_or_else(|| "—".to_string()),
                            bg: StateKind::from_name(&s.name).bg(),
                        },
                )
                .collect();

            let token = emit! {
                <div class="flex flex-col gap-4">
                    card(
                        card_header(
                            card_title("State timeline")
                            card_description(
                                "Each block is time spent in one state. Grey background means no data."
                            )
                        )
                        card_content(
                            <div
                                class="relative h-12 w-full overflow-hidden rounded-md bg-muted"
                            >
                                for b in &blocks {
                                    <div
                                        class=(format!("absolute inset-y-0 {}", b.kind.bg()))
                                        style=(b.style(&w))
                                        title=(b.title())
                                    ></div>
                                }
                            </div>
                            <div
                                class="mt-1 flex justify-between text-xs text-muted-foreground"
                            >
                                <span>(w.start_label())</span>
                                <span>(w.end_label())</span>
                            </div>
                            <div class="mt-3 flex flex-wrap gap-x-4 gap-y-1">
                                for k in StateKind::ALL {
                                    <span class="inline-flex items-center gap-1.5 text-xs">
                                        <span
                                            class=(format!(
                                                "inline-block size-2.5 rounded-sm {}",
                                                k.bg(),
                                            ))
                                        ></span>
                                        (k.label())
                                    </span>
                                }
                            </div>
                        )
                    )
                    <div class="grid gap-4 lg:grid-cols-3">
                        card(
                            card_header(
                                card_title("Time in state")
                                card_description("Share of the selected range")
                            )
                            card_content(
                                <div
                                    class="flex h-3 w-full overflow-hidden rounded-full bg-muted"
                                >
                                    for s in &shares {
                                        <div class=(s.bg) style=(format!("width:{:.2}%", s.pct))></div>
                                    }
                                </div>
                                <dl class="mt-4 flex flex-col gap-2 text-sm">
                                    for s in &shares {
                                        <div class="flex items-center justify-between gap-2">
                                            <dt class="inline-flex items-center gap-2">
                                                <span
                                                    class=(format!("inline-block size-2.5 rounded-sm {}", s.bg))
                                                ></span>
                                                (s.label)
                                            </dt>
                                            <dd class="tabular-nums text-muted-foreground">
                                                (format!("{:.0}% · {}", s.pct, human(s.secs)))
                                            </dd>
                                        </div>
                                    }
                                </dl>
                                <p class="mt-4 text-sm text-muted-foreground">
                                    "Faults: "
                                    (fault_line)
                                </p>
                            )
                        )
                        <div class="lg:col-span-2">
                            card(
                                card_header(
                                    card_title("State changes")
                                    card_description("Newest first, within the selected range")
                                )
                                card_content(
                                    <div class="overflow-x-auto">
                                        table(
                                            table_header(
                                                table_row(
                                                    table_head("Time")
                                                    table_head("State")
                                                    table_head("Reason")
                                                )
                                            )
                                            table_body(
                                                for r in &log {
                                                    table_row(
                                                        table_cell((&r.time))
                                                        table_cell(
                                                            <span
                                                                class=(format!(
                                                                    "mr-2 inline-block size-2.5 rounded-sm {}",
                                                                    r.bg,
                                                                ))
                                                            ></span>
                                                            (&r.name)
                                                        )
                                                        table_cell((&r.reason))
                                                    )
                                                }
                                            )
                                        )
                                    </div>
                                )
                            )
                        </div>
                    </div>
                </div>
            }?;

            if !connected(cx) {
                break Ok(token);
            }
            wait_for(&mut rx, Kind::State).await;
        }
    })
}
