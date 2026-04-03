use leptos::prelude::*;
use crate::models::DailyData;

#[component]
pub fn TrendCharts(data: ReadSignal<Vec<DailyData>>) -> impl IntoView {
    view! {
        <div>
            <BarChart title="HRV (ms)" data=data extract=|r: &DailyData| r.hrv_last_night color="#00d4aa" />
            <BarChart title="Resting Heart Rate (bpm)" data=data extract=|r: &DailyData| r.resting_heart_rate.map(|x| x as f64) color="#ff6b8a" />
            <BarChart title="Sleep Score" data=data extract=|r: &DailyData| r.sleep_score.map(|x| x as f64) color="#4a9eff" />
            <BarChart title="Body Battery Peak" data=data extract=|r: &DailyData| r.body_battery_high.map(|x| x as f64) color="#ffb347" />
            <BarChart title="Stress" data=data extract=|r: &DailyData| r.avg_stress.map(|x| x as f64) color="#ff6b8a" />
            <BarChart title="Steps" data=data extract=|r: &DailyData| r.steps.map(|x| x as f64) color="#00d4aa" />
            <BarChart title="Training Readiness" data=data extract=|r: &DailyData| r.training_readiness color="#4a9eff" />
            <BarChart title="VO2 Max" data=data extract=|r: &DailyData| r.vo2_max color="#00d4aa" />
        </div>
    }
}

#[component]
fn BarChart(
    title: &'static str,
    data: ReadSignal<Vec<DailyData>>,
    extract: fn(&DailyData) -> Option<f64>,
    color: &'static str,
) -> impl IntoView {
    let chart_data = move || {
        let d = data.get();
        let vals: Vec<(String, Option<f64>)> = d.iter()
            .map(|row| (row.date.clone(), extract(row)))
            .collect();

        let non_null: Vec<f64> = vals.iter().filter_map(|(_, v)| *v).collect();
        if non_null.is_empty() {
            return None;
        }

        let max = non_null.iter().cloned().fold(f64::MIN, f64::max);
        let avg = non_null.iter().sum::<f64>() / non_null.len() as f64;
        let show_every = if vals.len() > 60 { (vals.len() / 30).max(1) } else if vals.len() > 30 { 2 } else { 1 };

        let bars: Vec<(u32, String, String, String)> = vals.iter().enumerate().map(|(i, (date, val))| {
            let height = match val {
                Some(v) if max > 0.0 => (v / max * 100.0) as u32,
                _ => 0,
            };
            let tooltip = match val {
                Some(v) => {
                    let vs = if *v == v.floor() { format!("{:.0}", v) } else { format!("{:.1}", v) };
                    format!("{}: {}", date, vs)
                }
                None => format!("{}: --", date),
            };
            let label = if i % show_every == 0 { date.get(5..).unwrap_or("").to_string() } else { String::new() };
            (height, tooltip, label, if val.is_some() { color.to_string() } else { "var(--color-border)".to_string() })
        }).collect();

        let avg_str = if avg == avg.floor() { format!("{:.0}", avg) } else { format!("{:.1}", avg) };
        Some((bars, avg_str))
    };

    view! {
        <Show when=move || chart_data().is_some()>
            {move || chart_data().map(|(bars, avg_str)| view! {
                <div class="bg-surface border border-border rounded-lg p-5 mb-4">
                    <div class="flex justify-between items-center mb-3">
                        <span class="text-text text-sm">{title}</span>
                        <span class="text-dim text-sm">"avg: " {avg_str}</span>
                    </div>
                    <div class="flex items-end gap-[2px] h-32">
                        {bars.iter().map(|(height, tooltip, _, bg)| {
                            let style = format!("height: {}%; background: {}; min-width: 2px;", height, bg);
                            let tooltip = tooltip.clone();
                            view! {
                                <div
                                    class="flex-1 rounded-t-sm relative opacity-70 hover:opacity-100 transition-opacity group"
                                    style=style
                                >
                                    <div class="hidden group-hover:block absolute bottom-full left-1/2 -translate-x-1/2 bg-bg border border-border px-2 py-1 rounded text-[0.65rem] whitespace-nowrap z-10">
                                        {tooltip}
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <div class="flex gap-[2px] mt-1">
                        {bars.iter().map(|(_, _, label, _)| {
                            view! {
                                <span class="flex-1 text-center text-[0.5rem] text-dim overflow-hidden min-w-[2px]">
                                    {label.clone()}
                                </span>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            })}
        </Show>
    }
}
