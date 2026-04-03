use leptos::prelude::*;
use crate::models::*;
use crate::theme;

#[component]
pub fn RecentActivitiesSection(data: ReadSignal<Vec<DailyData>>) -> impl IntoView {
    let activities = move || {
        let d = data.get();
        let mut acts: Vec<Activity> = Vec::new();
        for day in d.iter().rev().take(7) {
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

    view! {
        <div>
            // Activity table
            {move || {
                let acts = activities();
                if acts.is_empty() {
                    return view! { <div class="text-dim text-sm">"No recent activities"</div> }.into_any();
                }
                view! {
                    <div class="bg-surface border border-border rounded-lg overflow-hidden mb-3">
                        <table class="w-full text-sm">
                            <thead>
                                <tr class="border-b border-border text-dim text-xs text-left">
                                    <th class="px-3 py-2">"Date"</th>
                                    <th class="px-3 py-2">"Activity"</th>
                                    <th class="px-3 py-2">"Type"</th>
                                    <th class="px-3 py-2 text-right">"Duration"</th>
                                    <th class="px-3 py-2 text-right">"Avg HR"</th>
                                    <th class="px-3 py-2 text-right">"Max HR"</th>
                                    <th class="px-3 py-2 text-right">"Calories"</th>
                                    <th class="px-3 py-2 text-right">"Distance"</th>
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
                                    let dist = a.distance_m.map(|d| format!("{:.1} km", d / 1000.0)).unwrap_or_default();
                                    let hr_color = a.avg_hr.map(|h| theme::hr_zone_color(h).to_string()).unwrap_or_else(|| theme::DIM.into());
                                    view! {
                                        <tr class="border-b border-border/50 hover:bg-border/20">
                                            <td class="px-3 py-2 text-dim">{date.get(5..).unwrap_or(&date).to_string()}</td>
                                            <td class="px-3 py-2 font-bold">{name}</td>
                                            <td class="px-3 py-2 text-dim">{atype}</td>
                                            <td class="px-3 py-2 text-right">{dur}</td>
                                            <td class="px-3 py-2 text-right" style=format!("color: {}", hr_color)>{avg_hr}</td>
                                            <td class="px-3 py-2 text-right text-warn">{max_hr}</td>
                                            <td class="px-3 py-2 text-right">{cal}</td>
                                            <td class="px-3 py-2 text-right text-info">{dist}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                }.into_any()
            }}

            // Activity type distribution (pie)
            {move || {
                let acts = activities();
                if acts.is_empty() { return view! { <div></div> }.into_any(); }
                let mut type_counts: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
                for a in &acts {
                    let t = a.activity_type.clone().unwrap_or_else(|| "Other".into());
                    *type_counts.entry(t).or_default() += 1.0;
                }
                let colors = [theme::CHART_GREEN, theme::CHART_BLUE, theme::CHART_ORANGE, theme::CHART_RED, theme::CHART_PURPLE, theme::CHART_YELLOW];
                let slices: Vec<crate::components::charts::pie_chart::PieSlice> = type_counts.into_iter().enumerate().map(|(i, (label, count))| {
                    crate::components::charts::pie_chart::PieSlice {
                        label, value: count,
                        color: colors[i % colors.len()].to_string(),
                    }
                }).collect();
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                        <crate::components::charts::pie_chart::PieChart title="Activity Types".into() slices=slices />
                    </div>
                }.into_any()
            }}
        </div>
    }
}
