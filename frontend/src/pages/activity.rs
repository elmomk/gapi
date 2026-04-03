use leptos::prelude::*;
use crate::state::AppState;
use crate::models::*;
use crate::theme;

#[component]
pub fn ActivityPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    let activities = move || {
        let d = state.daily_data.get();
        let mut acts: Vec<Activity> = Vec::new();
        for day in d.iter().rev() {
            if let Some(ref json) = day.activities_json {
                if let Ok(parsed) = serde_json::from_str::<Vec<Activity>>(json) {
                    for mut a in parsed {
                        a.date = Some(day.date.clone());
                        acts.push(a);
                    }
                }
            }
        }
        acts
    };

    // Activity type stats
    let type_stats = move || {
        let acts = activities();
        let mut counts: std::collections::HashMap<String, (i32, f64, f64)> = std::collections::HashMap::new(); // (count, total_duration, total_calories)
        for a in &acts {
            let t = a.activity_type.clone().unwrap_or_else(|| "Other".into());
            let entry = counts.entry(t).or_default();
            entry.0 += 1;
            entry.1 += a.duration_secs.unwrap_or(0.0);
            entry.2 += a.calories.unwrap_or(0) as f64;
        }
        counts
    };

    view! {
        <div class="animate-slide-up">
            <h1 class="page-title">"Activity"</h1>
            <p class="page-subtitle">"Workouts and exercises"</p>

            // Activity type summary cards
            {move || {
                let stats = type_stats();
                if stats.is_empty() { return view! { <div></div> }.into_any(); }
                let colors = [theme::CHART_GREEN, theme::CHART_BLUE, theme::CHART_ORANGE, theme::CHART_RED, theme::CHART_PURPLE, theme::CHART_YELLOW];
                view! {
                    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-3 mb-6">
                        {stats.into_iter().enumerate().map(|(i, (name, (count, dur, cal)))| {
                            let c = colors[i % colors.len()];
                            view! {
                                <div class="card">
                                    <div class="metric-label mb-1">{name}</div>
                                    <div class="metric-value text-xl" style=format!("color: {}", c)>{count}</div>
                                    <div class="text-dim text-xs mt-1">{theme::fmt_duration(dur)} " total"</div>
                                    <div class="text-dim text-xs">{format!("{:.0} cal", cal)}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}

            // Activity table
            {move || {
                let acts = activities();
                if acts.is_empty() { return view! { <div class="card text-dim">"No activities in this period"</div> }.into_any(); }
                view! {
                    <div class="card-flat overflow-x-auto mb-4">
                        <table class="min-w-[600px] w-full text-sm font-display">
                            <thead>
                                <tr class="border-b border-white/[0.06] text-dim text-xs text-left">
                                    <th class="px-3 py-3">"Date"</th>
                                    <th class="px-3 py-3">"Activity"</th>
                                    <th class="px-3 py-3">"Type"</th>
                                    <th class="px-3 py-3 text-right">"Duration"</th>
                                    <th class="px-3 py-3 text-right">"Avg HR"</th>
                                    <th class="px-3 py-3 text-right">"Max HR"</th>
                                    <th class="px-3 py-3 text-right">"Calories"</th>
                                    <th class="px-3 py-3 text-right">"Distance"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {acts.iter().map(|a| {
                                    let date = a.date.clone().unwrap_or_default();
                                    let name = a.name.clone().unwrap_or_else(|| "Activity".into());
                                    let atype = a.activity_type.clone().unwrap_or_default();
                                    let dur = a.duration_secs.map(|s| theme::fmt_duration(s)).unwrap_or_default();
                                    let avg_hr = a.avg_hr.map(|h| format!("{}", h)).unwrap_or_default();
                                    let max_hr = a.max_hr.map(|h| format!("{}", h)).unwrap_or_default();
                                    let cal = a.calories.map(|c| format!("{}", c)).unwrap_or_default();
                                    let dist = a.distance_m.map(|d| format!("{:.1}km", d / 1000.0)).unwrap_or_default();
                                    let hr_color = a.avg_hr.map(|h| theme::hr_zone_color(h).to_string()).unwrap_or_else(|| theme::DIM.into());
                                    view! {
                                        <tr class="border-b border-white/[0.03] hover:bg-white/[0.02] transition-colors">
                                            <td class="px-3 py-2.5 text-dim font-mono text-xs">{date.get(5..).unwrap_or(&date).to_string()}</td>
                                            <td class="px-3 py-2.5 font-semibold">{name}</td>
                                            <td class="px-3 py-2.5 text-dim">{atype}</td>
                                            <td class="px-3 py-2.5 text-right">{dur}</td>
                                            <td class="px-3 py-2.5 text-right" style=format!("color: {}", hr_color)>{avg_hr}</td>
                                            <td class="px-3 py-2.5 text-right text-warn">{max_hr}</td>
                                            <td class="px-3 py-2.5 text-right">{cal}</td>
                                            <td class="px-3 py-2.5 text-right text-info">{dist}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
