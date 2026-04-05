use leptos::prelude::*;
use chrono::Datelike;
use crate::state::AppState;
use crate::models::*;
use crate::theme;
use crate::components::charts::bar_chart::*;

fn training_heatmap(daily: &[DailyData]) -> Vec<(String, i64)> {
    daily.iter().map(|d| {
        (d.date.clone(), d.activities_count.unwrap_or(0))
    }).collect()
}

fn weekly_volume(daily: &[DailyData]) -> Vec<BarPoint> {
    let mut weeks: std::collections::BTreeMap<String, f64> = std::collections::BTreeMap::new();
    for d in daily {
        if let Some(ref json) = d.activities_json {
            if let Ok(acts) = serde_json::from_str::<Vec<serde_json::Value>>(json) {
                for a in &acts {
                    if let Some(vol) = a["total_volume_kg"].as_f64() {
                        if vol > 0.0 {
                            let week = &d.date[..7]; // YYYY-MM as rough grouping
                            *weeks.entry(week.to_string()).or_default() += vol;
                        }
                    }
                }
            }
        }
    }
    weeks.into_iter().map(|(k, v)| BarPoint { label: k, value: v, color: None }).collect()
}

#[component]
pub fn ActivityPage() -> impl IntoView {
    let state = expect_context::<AppState>();
    let (expanded_id, set_expanded_id) = signal(None::<i64>);

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
        let mut counts: std::collections::HashMap<String, (i32, f64, f64)> = std::collections::HashMap::new();
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

            // Training Consistency Heatmap
            {move || {
                let d = state.daily_data.get();
                let heatmap_data = training_heatmap(&d);
                if heatmap_data.is_empty() { return view! { <div></div> }.into_any(); }

                // Build a grid: 7 columns (Mon-Sun) x N weeks
                // Parse dates and arrange into week rows
                let mut cells: Vec<(String, i64, u32)> = Vec::new(); // (date, count, weekday 0=Mon)
                for (date, count) in &heatmap_data {
                    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                        let wd = parsed.weekday().num_days_from_monday();
                        cells.push((date.clone(), *count, wd));
                    }
                }

                if cells.is_empty() { return view! { <div></div> }.into_any(); }

                // Group into weeks (each week starts on Monday)
                let mut weeks: Vec<Vec<(String, i64)>> = Vec::new();
                let mut current_week: Vec<(String, i64)> = vec![("".to_string(), -1); 7];
                let mut last_week_num: Option<i32> = None;

                for (date, count, wd) in &cells {
                    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                        let week_num = parsed.iso_week().week() as i32 + parsed.iso_week().year() as i32 * 100;
                        if last_week_num.is_some() && last_week_num != Some(week_num) {
                            weeks.push(current_week);
                            current_week = vec![("".to_string(), -1); 7];
                        }
                        last_week_num = Some(week_num);
                        current_week[*wd as usize] = (date.clone(), *count);
                    }
                }
                weeks.push(current_week);

                let day_labels = ["M", "T", "W", "T", "F", "S", "S"];

                view! {
                    <div class="card mb-6">
                        <div class="text-text text-sm mb-3 font-display font-semibold">"Training Consistency"</div>
                        <div class="flex gap-1">
                            // Day labels
                            <div class="flex flex-col gap-[2px] mr-1">
                                {day_labels.iter().map(|l| {
                                    view! { <div class="text-[0.5rem] text-dim leading-none" style="height: 12px; display: flex; align-items: center;">{*l}</div> }
                                }).collect::<Vec<_>>()}
                            </div>
                            // Week columns
                            <div class="flex gap-[2px] flex-1 overflow-x-auto">
                                {weeks.into_iter().map(|week| {
                                    view! {
                                        <div class="flex flex-col gap-[2px]">
                                            {week.into_iter().map(|(date, count)| {
                                                let (bg, tip) = if count < 0 {
                                                    (format!("background: {}; opacity: 0.15;", theme::SURFACE), String::new())
                                                } else if count == 0 {
                                                    (format!("background: {}; opacity: 0.3;", theme::DIM), format!("{}: no activity", date))
                                                } else if count == 1 {
                                                    (format!("background: {}; opacity: 0.6;", theme::CHART_GREEN), format!("{}: {} activity", date, count))
                                                } else {
                                                    (format!("background: {};", theme::CHART_GREEN), format!("{}: {} activities", date, count))
                                                };
                                                view! {
                                                    <div class="rounded-[2px] relative group" style=format!("width: 12px; height: 12px; {}", bg)>
                                                        {if !tip.is_empty() {
                                                            view! { <div class="hidden group-hover:block absolute bottom-full left-1/2 -translate-x-1/2 bg-bg border border-border px-2 py-1 rounded text-[0.55rem] whitespace-nowrap z-20">{tip}</div> }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                        <div class="flex items-center gap-2 mt-2 text-[0.55rem] text-dim">
                            <span>"Less"</span>
                            <div class="rounded-[2px]" style=format!("width: 10px; height: 10px; background: {}; opacity: 0.3;", theme::DIM)></div>
                            <div class="rounded-[2px]" style=format!("width: 10px; height: 10px; background: {}; opacity: 0.6;", theme::CHART_GREEN)></div>
                            <div class="rounded-[2px]" style=format!("width: 10px; height: 10px; background: {};", theme::CHART_GREEN)></div>
                            <span>"More"</span>
                        </div>
                    </div>
                }.into_any()
            }}

            // Volume Trend (Progressive Overload)
            {move || {
                let d = state.daily_data.get();
                let vol_data = weekly_volume(&d);
                if vol_data.is_empty() { return view! { <div></div> }.into_any(); }
                view! {
                    <BarChart title="Weekly Training Volume".into() data=vol_data
                        color=theme::CHART_ORANGE.into() unit="kg".into() />
                }.into_any()
            }}

            // Activity list (cards, not table -- better for mobile + expandable)
            {move || {
                let acts = activities();
                if acts.is_empty() { return view! { <div class="card text-dim">"No activities in this period"</div> }.into_any(); }
                view! {
                    <div class="space-y-2">
                        {acts.into_iter().map(|a| {
                            let act_id = a.id.unwrap_or(0);
                            let date = a.date.clone().unwrap_or_default();
                            let name = a.name.clone().unwrap_or_else(|| "Activity".into());
                            let atype = a.activity_type.clone().unwrap_or_default();
                            let dur = a.duration_secs.map(|s| theme::fmt_duration(s)).unwrap_or_default();
                            let avg_hr = a.avg_hr.map(|h| format!("avg {} bpm", h)).unwrap_or_default();
                            let max_hr = a.max_hr.map(|h| format!("max {} bpm", h)).unwrap_or_default();
                            let cal = a.calories.map(|c| format!("{} cal", c)).unwrap_or_default();
                            let dist = a.distance_m.filter(|d| *d > 0.0).map(|d| format!("{:.1} km", d / 1000.0));
                            let hr_color = a.avg_hr.map(|h| theme::hr_zone_color(h).to_string()).unwrap_or_else(|| theme::DIM.into());
                            let sets_info = a.total_sets.map(|s| format!("{} sets", s));
                            let reps_info = a.total_reps.map(|r| format!("{} reps", r));
                            let volume_info = a.total_volume_kg.filter(|v| *v > 0.0).map(|v| format!("{:.0} kg", v));
                            let exercises = a.exercises.clone().unwrap_or_default();
                            let has_exercises = !exercises.is_empty();
                            let is_strength = atype.contains("strength") || atype.contains("gym") || atype.contains("weight");
                            let date_short = date.get(5..).unwrap_or(&date).to_string();

                            view! {
                                <div class="card cursor-pointer" on:click=move |_| {
                                    if has_exercises {
                                        set_expanded_id.set(if expanded_id.get() == Some(act_id) { None } else { Some(act_id) });
                                    }
                                }>
                                    // Main row
                                    <div class="flex justify-between items-start">
                                        <div>
                                            <div class="flex items-center gap-2">
                                                <span class="font-display font-semibold">{name.clone()}</span>
                                                {if has_exercises {
                                                    view! { <span class="text-accent text-xs">"▼"</span> }.into_any()
                                                } else {
                                                    view! { <span></span> }.into_any()
                                                }}
                                            </div>
                                            <div class="text-dim text-xs mt-0.5">{date_short} " · " {atype.clone()} " · " {dur}</div>
                                        </div>
                                        <div class="text-right text-sm">
                                            <div style=format!("color: {}", hr_color)>{avg_hr}</div>
                                            <div class="text-xs" style=format!("color: {}", theme::CHART_RED)>{max_hr}</div>
                                            <div class="text-dim text-xs">{cal}</div>
                                        </div>
                                    </div>

                                    // Stats row for strength
                                    {if is_strength {
                                        view! {
                                            <div class="flex gap-4 mt-2 text-xs">
                                                {sets_info.map(|s| view! { <span class="text-accent">{s}</span> })}
                                                {reps_info.map(|r| view! { <span class="text-info">{r}</span> })}
                                                {volume_info.map(|v| view! { <span class="text-stress">{v}</span> })}
                                            </div>
                                        }.into_any()
                                    } else {
                                        dist.map(|d| view! { <div class="text-info text-xs mt-1">{d}</div> }.into_any())
                                            .unwrap_or_else(|| view! { <span></span> }.into_any())
                                    }}

                                    // Expanded exercise detail
                                    <Show when=move || expanded_id.get() == Some(act_id) && has_exercises>
                                        <div class="mt-3 pt-3 border-t border-white/[0.06] animate-slide-up">
                                            {exercises.iter().map(|ex| {
                                                let ex_name = ex["exercise"].as_str().unwrap_or("Unknown").to_string();
                                                let sets = ex["sets"].as_array().cloned().unwrap_or_default();
                                                view! {
                                                    <div class="mb-3">
                                                        <div class="font-display font-semibold text-sm text-text mb-1">{ex_name}</div>
                                                        <div class="grid grid-cols-3 gap-1 text-xs">
                                                            <div class="text-dim font-semibold">"Set"</div>
                                                            <div class="text-dim font-semibold text-right">"Weight"</div>
                                                            <div class="text-dim font-semibold text-right">"Reps"</div>
                                                            {sets.iter().enumerate().map(|(i, set)| {
                                                                let weight = set["weight_kg"].as_f64().map(|w| format!("{:.1} kg", w)).unwrap_or_else(|| "-".into());
                                                                let reps = set["reps"].as_i64().map(|r| format!("{}", r)).unwrap_or_else(|| "-".into());
                                                                view! {
                                                                    <div class="text-dim">{format!("{}", i + 1)}</div>
                                                                    <div class="text-stress text-right">{weight}</div>
                                                                    <div class="text-accent text-right">{reps}</div>
                                                                }
                                                            }).collect::<Vec<_>>()}
                                                        </div>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </Show>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
