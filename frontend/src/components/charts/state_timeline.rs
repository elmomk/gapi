use leptos::prelude::*;

/// A segment in the timeline
#[derive(Clone)]
pub struct TimelineSegment {
    pub label: String,
    pub value: f64,
    pub color: String,
}

/// A row in a multi-row state timeline
#[derive(Clone)]
pub struct TimelineRow {
    pub label: String,
    pub segments: Vec<TimelineSegment>,
}

/// Horizontal colored block timeline
#[component]
pub fn StateTimeline(
    title: String,
    rows: Vec<TimelineRow>,
) -> impl IntoView {
    if rows.is_empty() {
        return view! { <div></div> }.into_any();
    }

    view! {
        <div class="bg-surface border border-border rounded-lg p-4 mb-3">
            <div class="text-text text-sm mb-2">{title}</div>
            {rows.iter().map(|row| {
                let total: f64 = row.segments.iter().map(|s| s.value.max(0.0)).sum::<f64>().max(0.001);
                let label = row.label.clone();
                view! {
                    <div class="flex items-center gap-2 mb-1">
                        <span class="text-dim text-[0.6rem] w-16 text-right flex-shrink-0">{label}</span>
                        <div class="flex flex-1 h-4 rounded overflow-hidden">
                            {row.segments.iter().map(|seg| {
                                let pct = seg.value / total * 100.0;
                                let style = format!("width: {}%; background: {};", pct, seg.color);
                                let tip = format!("{}: {}", seg.label, crate::theme::fmt_val(seg.value));
                                view! {
                                    <div class="relative group" style=style title=tip.clone()>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }.into_any()
}
