use leptos::prelude::*;
use crate::state::AppState;
use crate::theme;
use crate::components::charts::gauge::Gauge;
use crate::components::charts::state_timeline::*;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    view! {
        <div class="animate-slide-up">
            <h1 class="page-title">"Dashboard"</h1>
            <p class="page-subtitle">"Today's health overview"</p>

            // Vitals grid
            {move || state.vitals.get().map(|v| {
                view! {
                    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
                        <VitalCard label="HRV" value=v.hrv_last_night unit="ms" baseline=v.baseline_hrv higher_is_better=true color=theme::CHART_GREEN />
                        <VitalCard label="Resting HR" value=v.resting_heart_rate.map(|x| x as f64) unit="bpm" baseline=v.baseline_rhr higher_is_better=false color=theme::CHART_RED />
                        <VitalCard label="Sleep Score" value=v.sleep_score.map(|x| x as f64) unit="" baseline=v.baseline_sleep higher_is_better=true color=theme::CHART_PURPLE />
                        <VitalCard label="Sleep" value=v.sleep_hours unit="hrs" baseline=None higher_is_better=true color=theme::CHART_BLUE />
                    </div>
                    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
                        <VitalCard label="Stress" value=v.avg_stress.map(|x| x as f64) unit="" baseline=v.baseline_stress higher_is_better=false color=theme::STRESS_MEDIUM />
                        <VitalCard label="Body Battery" value=v.body_battery_high.map(|x| x as f64) unit="%" baseline=v.baseline_battery higher_is_better=true color=theme::BB_CHARGED />
                        <VitalCard label="Readiness" value=v.training_readiness unit="" baseline=None higher_is_better=true color=theme::CHART_BLUE />
                        <VitalCard label="Steps" value=v.steps.map(|x| x as f64) unit="" baseline=None higher_is_better=true color=theme::CHART_GREEN />
                    </div>

                    // Gauges row
                    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
                        <Gauge title="RHR".into() value=v.resting_heart_rate.map(|x| x as f64)
                            min=40.0 max=100.0 unit="bpm".into()
                            thresholds=vec![(40.0, theme::CHART_BLUE.into()), (60.0, theme::GOOD.into()), (70.0, theme::CHART_YELLOW.into()), (80.0, theme::WARN.into())] />
                        <Gauge title="Steps".into() value=v.steps.map(|x| x as f64)
                            min=0.0 max=15000.0 unit="".into()
                            thresholds=vec![(0.0, theme::WARN.into()), (5000.0, theme::CHART_ORANGE.into()), (10000.0, theme::GOOD.into())] />
                        <Gauge title="Body Battery".into() value=v.body_battery_high.map(|x| x as f64)
                            min=0.0 max=100.0 unit="%".into()
                            thresholds=vec![(0.0, theme::WARN.into()), (30.0, theme::CHART_YELLOW.into()), (60.0, theme::GOOD.into())] />
                        <Gauge title="Sleep".into() value=v.sleep_hours
                            min=0.0 max=10.0 unit="hrs".into()
                            thresholds=vec![(0.0, theme::WARN.into()), (5.0, theme::CHART_YELLOW.into()), (7.0, theme::GOOD.into())] />
                    </div>
                }
            })}

            // Month at a Glance
            {move || {
                let d = state.daily_data.get();
                if d.len() < 7 { return view! { <div></div> }.into_any(); }
                let make_row = |label: &str, extract: fn(&crate::models::DailyData) -> Option<f64>, low: &str, high: &str| -> TimelineRow {
                    let vals: Vec<f64> = d.iter().filter_map(|d| extract(d)).collect();
                    let max = vals.iter().cloned().fold(f64::MIN, f64::max).max(0.001);
                    TimelineRow {
                        label: label.to_string(),
                        segments: d.iter().map(|day| {
                            let v = extract(day).unwrap_or(0.0);
                            TimelineSegment { label: day.date.clone(), value: 1.0, color: theme::lerp_color(low, high, (v / max).clamp(0.0, 1.0)) }
                        }).collect(),
                    }
                };
                view! {
                    <StateTimeline title="Month at a Glance".into() rows=vec![
                        make_row("Steps", |d| d.steps.map(|v| v as f64), theme::BG, theme::CHART_GREEN),
                        make_row("RHR", |d| d.resting_heart_rate.map(|v| v as f64), theme::CHART_GREEN, theme::CHART_RED),
                        make_row("HRV", |d| d.hrv_last_night, theme::BG, theme::CHART_GREEN),
                        make_row("Sleep", |d| d.sleep_score.map(|v| v as f64), theme::CHART_RED, theme::CHART_BLUE),
                        make_row("Stress", |d| d.avg_stress.map(|v| v as f64), theme::CHART_GREEN, theme::CHART_RED),
                        make_row("Battery", |d| d.body_battery_high.map(|v| v as f64), theme::CHART_RED, theme::CHART_GREEN),
                    ] />
                }.into_any()
            }}

            // Today's activities
            {move || {
                let d = state.daily_data.get();
                let today_acts = d.last().and_then(|day| day.activities_json.as_ref())
                    .and_then(|j| serde_json::from_str::<Vec<crate::models::Activity>>(j).ok())
                    .unwrap_or_default();
                if today_acts.is_empty() { return view! { <div></div> }.into_any(); }
                view! {
                    <h2 class="text-sm font-display font-semibold text-text mt-6 mb-3">"Today's Activities"</h2>
                    <div class="grid gap-2">
                        {today_acts.into_iter().map(|a| {
                            let name = a.name.unwrap_or_else(|| "Activity".into());
                            let atype = a.activity_type.unwrap_or_default();
                            let dur = a.duration_secs.map(|s| theme::fmt_duration(s)).unwrap_or_default();
                            let hr = a.avg_hr.map(|h| format!("{} bpm", h)).unwrap_or_default();
                            let cal = a.calories.map(|c| format!("{} cal", c)).unwrap_or_default();
                            view! {
                                <div class="card flex justify-between items-center">
                                    <div>
                                        <div class="font-display font-semibold">{name}</div>
                                        <div class="text-dim text-xs">{atype} " " {dur}</div>
                                    </div>
                                    <div class="text-right text-sm">
                                        <div class="text-heart">{hr}</div>
                                        <div class="text-dim">{cal}</div>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn VitalCard(
    label: &'static str,
    value: Option<f64>,
    unit: &'static str,
    baseline: Option<f64>,
    higher_is_better: bool,
    color: &'static str,
) -> impl IntoView {
    let display = match value {
        Some(v) if v == v.floor() && v.abs() < 100000.0 => format!("{:.0}", v),
        Some(v) => format!("{:.1}", v),
        None => "\u{2014}".to_string(),
    };

    let (delta_class, delta_text) = match (value, baseline) {
        (Some(today), Some(base)) if base > 0.0 => {
            let pct = ((today - base) / base * 100.0) as i64;
            let better = if higher_is_better { today >= base * 0.9 } else { today <= base * 1.1 };
            let cls = if better { "metric-delta up" } else { "metric-delta down" };
            let arrow = if pct >= 0 { "\u{2191}" } else { "\u{2193}" };
            (cls, format!("{}{:.0}%", arrow, pct.abs()))
        }
        _ => ("metric-delta text-dim", "".to_string()),
    };

    view! {
        <div class="card group">
            <div class="metric-label mb-2">{label}</div>
            <div class="flex items-baseline gap-1.5">
                <span class="metric-value" style=format!("color: {}", color)>{display}</span>
                <span class="text-dim text-sm font-display">{unit}</span>
            </div>
            <div class=delta_class>{delta_text}</div>
        </div>
    }
}
