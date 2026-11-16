use topcoat::Result;
use topcoat::asset::asset;
use topcoat::context::Cx;
use topcoat::router::page;
use topcoat::runtime::{Event, signal};
use topcoat::view::{View, view};

use crate::features::Range;
use crate::features::moisture::{moisture_charts, moisture_tile};
use crate::features::state::{state_section, state_tile};
use crate::features::stepper::{stepper_charts, stepper_tile};

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    let range = signal(cx, || Range::default().slug().to_owned());

    Ok(view! {
        <div class="mx-auto flex max-w-6xl flex-col gap-4 p-4 sm:p-6">
            <header class="flex flex-wrap items-center justify-between gap-3">
                <h1 class="text-2xl font-semibold">"Plant"</h1>
                <label class="flex items-center gap-2 text-sm">
                    <span class="text-muted-foreground">"Range"</span>
                    <select
                        class="rounded-md border bg-background px-2 py-1"
                        @change=$(|e: Event| range.set(e.target.value))
                    >
                        for r in Range::ALL {
                            <option value=(r.slug()) selected=(r == Range::default())>
                                (r.label())
                            </option>
                        }
                    </select>
                </label>
            </header>

            <div class="grid gap-4 md:grid-cols-3">
                state_tile()
                moisture_tile()
                stepper_tile()
            </div>

            state_section(range: $(range.get()))
            moisture_charts(range: $(range.get()))
            stepper_charts(range: $(range.get()))

            <script src=(asset!("./dashboard.js"))></script>
        </div>
    })
}
