use leptos::prelude::*;
use crate::models::DailyData;

#[component]
pub fn Activities(data: ReadSignal<Vec<DailyData>>) -> impl IntoView {
    let activities = move || {
        let d = data.get();
        // Get today's entry (last in the sorted vec)
        let today = d.last().and_then(|entry| entry.activities_json.as_ref());
        match today {
            Some(json) => serde_json::from_str::<Vec<serde_json::Value>>(json).unwrap_or_default(),
            None => Vec::new(),
        }
    };

    view! {
        <div>
            {move || {
                let acts = activities();
                if acts.is_empty() {
                    return view! {
                        <div class="text-dim text-sm">"No activities today"</div>
                    }.into_any();
                }
                view! {
                    <div>
                        {acts.into_iter().map(|a| {
                            let name = a["name"].as_str().or(a["type"].as_str()).unwrap_or("Activity").to_string();
                            let activity_type = a["activityType"].as_str().unwrap_or("").to_string();
                            let duration = a["duration_secs"].as_f64()
                                .map(|s| format!("{}min", (s / 60.0).round() as i64))
                                .unwrap_or_default();
                            let calories = a["calories"].as_i64()
                                .map(|c| format!("{c} cal"))
                                .unwrap_or_default();
                            let avg_hr = a["avg_hr"].as_i64()
                                .map(|h| format!("{h} bpm"))
                                .unwrap_or_default();
                            let stats = [avg_hr, calories].into_iter()
                                .filter(|s| !s.is_empty())
                                .collect::<Vec<_>>()
                                .join(" · ");

                            view! {
                                <div class="bg-surface border border-border rounded-lg px-4 py-3 mb-2 flex justify-between items-center">
                                    <div>
                                        <div class="font-bold text-sm">{name}</div>
                                        <div class="text-dim text-xs">{activity_type} " " {duration}</div>
                                    </div>
                                    <div class="text-sm text-right">{stats}</div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
