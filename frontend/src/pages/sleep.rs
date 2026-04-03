use leptos::prelude::*;
use crate::state::AppState;
use crate::theme;
use crate::models::DailyData;
use crate::components::charts::timeseries::*;
use crate::components::charts::bar_chart::*;
use crate::components::charts::pie_chart::*;
use crate::components::charts::gauge::Gauge;
use crate::components::charts::state_timeline::*;

fn compute_sleep_debt(daily: &[DailyData], target_hours: f64) -> (f64, Vec<(String, f64)>) {
    let target_secs = target_hours * 3600.0;
    let mut debt = 0.0;
    let mut daily_debts = Vec::new();
    for d in daily.iter().rev().take(14).rev() {
        let slept = d.sleep_duration_secs.unwrap_or(0) as f64;
        debt += target_secs - slept;
        daily_debts.push((d.date.clone(), debt / 3600.0));
    }
    (debt / 3600.0, daily_debts)
}

fn sleep_efficiency(daily: &[DailyData]) -> Option<f64> {
    let last = daily.last()?;
    let total_secs = last.sleep_duration_secs? as f64;
    let awake_secs = last.awake_secs.unwrap_or(0) as f64;
    let bed_time = total_secs + awake_secs;
    if bed_time > 0.0 { Some(total_secs / bed_time * 100.0) } else { None }
}

fn sleep_stage_percentages(daily: &[DailyData]) -> Option<(f64, f64, f64, f64)> {
    let last = daily.last()?;
    let deep = last.deep_sleep_secs? as f64;
    let light = last.light_sleep_secs? as f64;
    let rem = last.rem_sleep_secs? as f64;
    let awake = last.awake_secs.unwrap_or(0) as f64;
    let total = deep + light + rem + awake;
    if total > 0.0 {
        Some((deep / total * 100.0, light / total * 100.0, rem / total * 100.0, awake / total * 100.0))
    } else {
        None
    }
}

#[component]
pub fn SleepPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    view! {
        <div class="animate-slide-up">
            <h1 class="page-title">"Sleep"</h1>
            <p class="page-subtitle">"Sleep quality and patterns"</p>

            // Sleep Debt Card
            {move || {
                let daily = state.daily_data.get();
                if daily.is_empty() { return view! { <div></div> }.into_any(); }
                let (total_debt, daily_debts) = compute_sleep_debt(&daily, 8.0);
                let debt_color = if total_debt <= 2.0 { theme::GOOD } else if total_debt <= 5.0 { theme::CHART_YELLOW } else { theme::WARN };
                let max_abs = daily_debts.iter().map(|(_, v)| v.abs()).fold(0.001_f64, f64::max);
                view! {
                    <div class="card mb-6" style=format!("border-left: 3px solid {}", debt_color)>
                        <div class="metric-label mb-1">"Sleep Debt (14-day, target 8h)"</div>
                        <div class="flex items-baseline gap-2 mb-2">
                            <span class="text-2xl font-display font-bold" style=format!("color: {}", debt_color)>
                                {format!("{:.1}h", total_debt)}
                            </span>
                            <span class="text-dim text-sm font-display">
                                {if total_debt > 0.0 { "deficit" } else { "surplus" }}
                            </span>
                        </div>
                        <div class="flex items-end gap-px" style="height: 32px">
                            {daily_debts.into_iter().map(|(_date, val)| {
                                let h = ((val.abs() / max_abs) * 28.0).max(2.0);
                                let c = if val > 0.0 { theme::WARN } else { theme::GOOD };
                                view! {
                                    <div style=format!("width: 100%; height: {:.0}px; background: {}; border-radius: 2px 2px 0 0; opacity: 0.7", h, c)></div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                }.into_any()
            }}

            // Today's sleep summary
            {move || state.vitals.get().map(|v| view! {
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
                    <Gauge title="Sleep Score".into() value=v.sleep_score.map(|x| x as f64)
                        min=0.0 max=100.0 unit="".into()
                        thresholds=vec![(0.0, theme::WARN.into()), (60.0, theme::CHART_YELLOW.into()), (80.0, theme::GOOD.into())] />
                    <Gauge title="Duration".into() value=v.sleep_hours
                        min=0.0 max=10.0 unit="hrs".into()
                        thresholds=vec![(0.0, theme::WARN.into()), (5.0, theme::CHART_YELLOW.into()), (7.0, theme::GOOD.into())] />
                    <Gauge title="HRV".into() value=v.hrv_last_night
                        min=0.0 max=150.0 unit="ms".into()
                        thresholds=vec![(0.0, theme::WARN.into()), (30.0, theme::CHART_YELLOW.into()), (50.0, theme::GOOD.into())] />
                    <Gauge title="Resting HR".into() value=v.resting_heart_rate.map(|x| x as f64)
                        min=40.0 max=100.0 unit="bpm".into()
                        thresholds=vec![(40.0, theme::CHART_BLUE.into()), (60.0, theme::GOOD.into()), (70.0, theme::WARN.into())] />
                </div>
            })}

            // Sleep efficiency & stage targets
            {move || {
                let d = state.daily_data.get();
                let eff = sleep_efficiency(&d);
                let stages = sleep_stage_percentages(&d);
                if eff.is_none() && stages.is_none() { return view! { <div></div> }.into_any(); }
                view! {
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 mb-6">
                        // Sleep efficiency card
                        {eff.map(|e| {
                            let color = if e >= 90.0 { theme::GOOD } else if e >= 80.0 { theme::CHART_YELLOW } else { theme::WARN };
                            let bar_width = format!("{}%", e.min(100.0));
                            view! {
                                <div class="card">
                                    <div class="metric-label mb-1">"Sleep Efficiency"</div>
                                    <div class="metric-value text-xl" style=format!("color: {}", color)>{format!("{:.0}%", e)}</div>
                                    <div class="mt-2 h-1.5 rounded-full" style=format!("background: {}33", theme::DIM)>
                                        <div class="h-full rounded-full transition-all" style=format!("width: {}; background: {}", bar_width, color)></div>
                                    </div>
                                    <div class="text-dim text-xs mt-1">"Time asleep vs time in bed"</div>
                                </div>
                            }
                        })}
                        // Stage percentage cards
                        {stages.map(|(deep, _light, rem, _awake)| {
                            let deep_color = if deep >= 15.0 && deep <= 20.0 { theme::GOOD } else if deep >= 12.0 || deep <= 23.0 { theme::CHART_YELLOW } else { theme::WARN };
                            let rem_color = if rem >= 20.0 && rem <= 25.0 { theme::GOOD } else if rem >= 17.0 || rem <= 28.0 { theme::CHART_YELLOW } else { theme::WARN };
                            let deep_label = if deep >= 15.0 && deep <= 20.0 { "On target" } else if deep < 15.0 { "Below target" } else { "Above target" };
                            let rem_label = if rem >= 20.0 && rem <= 25.0 { "On target" } else if rem < 20.0 { "Below target" } else { "Above target" };
                            view! {
                                <div class="card">
                                    <div class="metric-label mb-1">"Deep Sleep"</div>
                                    <div class="metric-value text-xl" style=format!("color: {}", deep_color)>{format!("{:.0}%", deep)}</div>
                                    <div class="text-dim text-xs mt-1">"Target: 15-20%"</div>
                                    <div class="text-xs mt-0.5" style=format!("color: {}", deep_color)>{deep_label}</div>
                                </div>
                                <div class="card">
                                    <div class="metric-label mb-1">"REM Sleep"</div>
                                    <div class="metric-value text-xl" style=format!("color: {}", rem_color)>{format!("{:.0}%", rem)}</div>
                                    <div class="text-dim text-xs mt-1">"Target: 20-25%"</div>
                                    <div class="text-xs mt-0.5" style=format!("color: {}", rem_color)>{rem_label}</div>
                                </div>
                            }
                        })}
                    </div>
                }.into_any()
            }}

            // Sleep intraday
            {move || {
                let sl = state.intraday_sleep.get();
                let hrv = state.intraday_hrv.get();
                if sl.is_empty() && hrv.is_empty() { return view! { <div class="card text-dim text-sm">"No intraday sleep data for today"</div> }.into_any(); }
                let mut series = vec![];
                let hr_pts: Vec<(f64, f64)> = sl.iter().enumerate().filter_map(|(i, e)| e.hr.map(|h| (i as f64, h as f64))).collect();
                if !hr_pts.is_empty() { series.push(Series { label: "Sleep HR".into(), points: hr_pts, color: theme::CHART_RED.into(), fill: false }); }
                if !hrv.is_empty() {
                    series.push(Series { label: "HRV".into(), points: hrv.iter().enumerate().map(|(i, h)| (i as f64, h.hrv_value)).collect(), color: theme::CHART_ORANGE.into(), fill: false });
                }
                view! { <TimeseriesChart title="Sleep HR & HRV".into() series=series /> }.into_any()
            }}

            // Sleep stages timeline
            {move || {
                let sl = state.intraday_sleep.get();
                if sl.is_empty() { return view! { <div></div> }.into_any(); }
                // Group consecutive stages into segments
                let mut segments: Vec<TimelineSegment> = Vec::new();
                for epoch in sl.iter() {
                    let stage = epoch.stage.as_deref().unwrap_or("unknown");
                    let (label, color) = match stage {
                        "deep" => ("Deep", theme::SLEEP_DEEP),
                        "light" => ("Light", theme::SLEEP_LIGHT),
                        "rem" => ("REM", theme::SLEEP_REM),
                        "awake" => ("Awake", theme::SLEEP_AWAKE),
                        _ => ("Unknown", theme::DIM),
                    };
                    // Merge with previous segment if same stage
                    if let Some(last) = segments.last_mut() {
                        if last.label == label {
                            last.value += 1.0;
                            continue;
                        }
                    }
                    segments.push(TimelineSegment {
                        label: label.to_string(),
                        value: 1.0,
                        color: color.to_string(),
                    });
                }
                let row = TimelineRow {
                    label: "Stages".to_string(),
                    segments,
                };
                view! {
                    <StateTimeline title="Sleep Stage Timeline".into() rows=vec![row] />
                }.into_any()
            }}

            // Long-term sleep charts
            <h2 class="text-sm font-display font-semibold text-text mt-8 mb-3">"Trends"</h2>
            {move || {
                let d = state.daily_data.get();
                if d.is_empty() { return view! { <div></div> }.into_any(); }

                let sleep_stacked: Vec<StackedBarPoint> = d.iter().map(|day| StackedBarPoint {
                    label: day.date.clone(),
                    segments: vec![
                        (day.deep_sleep_secs.unwrap_or(0) as f64, theme::SLEEP_DEEP.into()),
                        (day.light_sleep_secs.unwrap_or(0) as f64, theme::SLEEP_LIGHT.into()),
                        (day.rem_sleep_secs.unwrap_or(0) as f64, theme::SLEEP_REM.into()),
                        (day.awake_secs.unwrap_or(0) as f64, theme::SLEEP_AWAKE.into()),
                    ],
                }).collect();

                let score_data: Vec<BarPoint> = d.iter().map(|d| BarPoint {
                    label: d.date.clone(), value: d.sleep_score.unwrap_or(0) as f64, color: None,
                }).collect();

                let days_with_sleep = d.iter().filter(|d| d.deep_sleep_secs.is_some()).count().max(1) as f64;
                let total_deep: f64 = d.iter().filter_map(|d| d.deep_sleep_secs).sum::<i64>() as f64 / days_with_sleep;
                let total_light: f64 = d.iter().filter_map(|d| d.light_sleep_secs).sum::<i64>() as f64 / days_with_sleep;
                let total_rem: f64 = d.iter().filter_map(|d| d.rem_sleep_secs).sum::<i64>() as f64 / days_with_sleep;
                let total_awake: f64 = d.iter().filter_map(|d| d.awake_secs).sum::<i64>() as f64 / days_with_sleep;

                view! {
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                        <div class="md:col-span-2">
                            <StackedBarChart title="Sleep Stages".into() data=sleep_stacked
                                legend=vec![("Deep".into(), theme::SLEEP_DEEP.into()), ("Light".into(), theme::SLEEP_LIGHT.into()), ("REM".into(), theme::SLEEP_REM.into()), ("Awake".into(), theme::SLEEP_AWAKE.into())] />
                        </div>
                        <PieChart title="Average Sleep Breakdown".into()
                            slices=vec![
                                PieSlice { label: "Deep".into(), value: total_deep, color: theme::SLEEP_DEEP.into() },
                                PieSlice { label: "Light".into(), value: total_light, color: theme::SLEEP_LIGHT.into() },
                                PieSlice { label: "REM".into(), value: total_rem, color: theme::SLEEP_REM.into() },
                                PieSlice { label: "Awake".into(), value: total_awake, color: theme::SLEEP_AWAKE.into() },
                            ]
                            format_fn=theme::fmt_hours />
                    </div>
                    <BarChart title="Sleep Score".into() data=score_data color=theme::CHART_BLUE.into()
                        thresholds=vec![(80.0, theme::GOOD.into())] />
                }.into_any()
            }}
        </div>
    }
}
