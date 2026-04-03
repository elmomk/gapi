use leptos::prelude::*;

/// A data point for a bar chart
#[derive(Clone)]
pub struct BarPoint {
    pub label: String,
    pub value: f64,
    pub color: Option<String>,
}

/// A stacked bar entry
#[derive(Clone)]
pub struct StackedBarPoint {
    pub label: String,
    pub segments: Vec<(f64, String)>, // (value, color)
}

/// Simple bar chart with optional threshold coloring
#[component]
pub fn BarChart(
    title: String,
    data: Vec<BarPoint>,
    #[prop(optional)] unit: String,
    #[prop(optional)] color: String,
    #[prop(optional)] thresholds: Vec<(f64, String)>, // (value, color) sorted ascending
    #[prop(optional)] height: Option<f64>,
) -> impl IntoView {
    let h = height.unwrap_or(160.0);
    let default_color = if color.is_empty() { crate::theme::ACCENT.to_string() } else { color };

    if data.is_empty() {
        return view! { <div></div> }.into_any();
    }

    let vals: Vec<f64> = data.iter().map(|p| p.value).collect();
    let max = vals.iter().cloned().fold(f64::MIN, f64::max).max(0.001);
    let avg = vals.iter().sum::<f64>() / vals.len() as f64;
    let avg_str = crate::theme::fmt_val(avg);

    let show_every = if data.len() > 90 { (data.len() / 15).max(1) } else if data.len() > 45 { (data.len() / 10).max(1) } else if data.len() > 20 { 3 } else { 1 };

    let get_color = move |val: f64, custom: &Option<String>| -> String {
        if let Some(c) = custom { return c.clone(); }
        if !thresholds.is_empty() {
            let mut c = default_color.clone();
            for (threshold, tc) in &thresholds {
                if val >= *threshold { c = tc.clone(); }
            }
            return c;
        }
        default_color.clone()
    };

    let bars: Vec<_> = data.iter().enumerate().map(|(i, p)| {
        let pct = (p.value / max * 100.0).max(0.0) as u32;
        let c = get_color(p.value, &p.color);
        let tip = format!("{}: {} {}", p.label, crate::theme::fmt_val(p.value), unit);
        let label = if i % show_every == 0 { p.label.get(5..).unwrap_or(&p.label).to_string() } else { String::new() };
        (pct, c, tip, label)
    }).collect();

    view! {
        <div class="bg-surface border border-border rounded-lg p-4 mb-3">
            <div class="flex justify-between items-center mb-2">
                <span class="text-text text-sm">{title}</span>
                <span class="text-dim text-xs">"avg: " {avg_str} " " {unit.clone()}</span>
            </div>
            <div class="flex items-end gap-[1px]" style=format!("height: {}px", h)>
                {bars.iter().map(|(pct, color, tip, _)| {
                    let style = format!("height: {}%; background: {}; min-width: 2px;", pct, color);
                    let tip = tip.clone();
                    view! {
                        <div class="flex-1 rounded-t-sm relative opacity-70 hover:opacity-100 transition-opacity group" style=style>
                            <div class="hidden group-hover:block absolute bottom-full left-1/2 -translate-x-1/2 bg-bg border border-border px-2 py-1 rounded text-[0.6rem] whitespace-nowrap z-20">{tip}</div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
            <div class="flex gap-[1px] mt-1">
                {bars.iter().map(|(_, _, _, label)| {
                    view! { <span class="flex-1 text-center text-[0.6rem] text-dim overflow-hidden">{label.clone()}</span> }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }.into_any()
}

/// Stacked bar chart
#[component]
pub fn StackedBarChart(
    title: String,
    data: Vec<StackedBarPoint>,
    #[prop(optional)] unit: String,
    #[prop(optional)] legend: Vec<(String, String)>, // (label, color)
    #[prop(optional)] height: Option<f64>,
) -> impl IntoView {
    let h = height.unwrap_or(160.0);

    if data.is_empty() {
        return view! { <div></div> }.into_any();
    }

    let max: f64 = data.iter().map(|p| p.segments.iter().map(|(v, _)| *v).sum::<f64>()).fold(f64::MIN, f64::max).max(0.001);
    let show_every = if data.len() > 90 { (data.len() / 15).max(1) } else if data.len() > 45 { (data.len() / 10).max(1) } else if data.len() > 20 { 3 } else { 1 };

    view! {
        <div class="bg-surface border border-border rounded-lg p-4 mb-3">
            <div class="flex justify-between items-center mb-2">
                <span class="text-text text-sm">{title}</span>
                {if !legend.is_empty() {
                    view! {
                        <div class="flex gap-3">
                            {legend.iter().map(|(label, color)| {
                                let c = color.clone();
                                let l = label.clone();
                                view! {
                                    <span class="text-[0.6rem] flex items-center gap-1">
                                        <span class="w-2 h-2 rounded-sm inline-block" style=format!("background: {}", c)></span>
                                        {l}
                                    </span>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
            </div>
            <div class="flex items-end gap-[1px]" style=format!("height: {}px", h)>
                {data.iter().enumerate().map(|(_, p)| {
                    let total: f64 = p.segments.iter().map(|(v, _)| *v).sum();
                    let total_pct = (total / max * 100.0).max(0.0);
                    let tip = format!("{}: {} {}", p.label, crate::theme::fmt_val(total), unit);
                    view! {
                        <div class="flex-1 flex flex-col-reverse relative group" style=format!("height: {}%", total_pct)>
                            {p.segments.iter().map(|(v, c)| {
                                let seg_pct = if total > 0.0 { v / total * 100.0 } else { 0.0 };
                                let style = format!("height: {}%; background: {};", seg_pct, c);
                                view! { <div style=style></div> }
                            }).collect::<Vec<_>>()}
                            <div class="hidden group-hover:block absolute bottom-full left-1/2 -translate-x-1/2 bg-bg border border-border px-2 py-1 rounded text-[0.6rem] whitespace-nowrap z-20">{tip}</div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
            <div class="flex gap-[1px] mt-1">
                {data.iter().enumerate().map(|(i, p)| {
                    let label = if i % show_every == 0 { p.label.get(5..).unwrap_or(&p.label).to_string() } else { String::new() };
                    view! { <span class="flex-1 text-center text-[0.6rem] text-dim overflow-hidden">{label}</span> }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }.into_any()
}
