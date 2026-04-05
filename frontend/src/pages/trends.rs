use leptos::prelude::*;
use crate::state::AppState;
use crate::theme;
use crate::components::charts::bar_chart::*;
use crate::components::charts::timeseries::*;
use crate::models::DailyData;

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len()) as f64;
    if n < 3.0 { return 0.0; }
    let x = &x[..n as usize];
    let y = &y[..n as usize];
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;
    let num: f64 = x.iter().zip(y).map(|(a, b)| (a - mean_x) * (b - mean_y)).sum();
    let den_x: f64 = x.iter().map(|a| (a - mean_x).powi(2)).sum::<f64>().sqrt();
    let den_y: f64 = y.iter().map(|b| (b - mean_y).powi(2)).sum::<f64>().sqrt();
    if den_x * den_y == 0.0 { 0.0 } else { num / (den_x * den_y) }
}

fn correlation_strength(r: f64) -> (&'static str, &'static str) {
    let abs_r = r.abs();
    let strength = if abs_r >= 0.7 { "strong" } else if abs_r >= 0.4 { "moderate" } else { "weak" };
    let direction = if r > 0.0 { "positive" } else { "negative" };
    let color = if abs_r < 0.4 {
        theme::DIM
    } else if r > 0.0 {
        theme::GOOD
    } else {
        theme::WARN
    };
    let label = match (strength, direction) {
        ("strong", "positive") => "strong positive",
        ("strong", "negative") => "strong negative",
        ("moderate", "positive") => "moderate positive",
        ("moderate", "negative") => "moderate negative",
        _ => "weak",
    };
    (label, color)
}

struct CorrelationPair {
    label: String,
    r: f64,
}

fn compute_correlations(daily: &[DailyData]) -> Vec<CorrelationPair> {
    let mut results = Vec::new();

    // Sleep score -> next-day stress
    {
        let mut sleep_scores = Vec::new();
        let mut next_stress = Vec::new();
        for i in 0..daily.len().saturating_sub(1) {
            if let (Some(ss), Some(st)) = (daily[i].sleep_score, daily[i + 1].avg_stress) {
                sleep_scores.push(ss as f64);
                next_stress.push(st as f64);
            }
        }
        let r = pearson(&sleep_scores, &next_stress);
        results.push(CorrelationPair { label: "Sleep Score -> Next-Day Stress".into(), r });
    }

    // HRV -> training readiness
    {
        let mut hrvs = Vec::new();
        let mut readiness = Vec::new();
        for d in daily {
            if let (Some(h), Some(tr)) = (d.hrv_last_night, d.training_readiness) {
                hrvs.push(h);
                readiness.push(tr);
            }
        }
        let r = pearson(&hrvs, &readiness);
        results.push(CorrelationPair { label: "HRV -> Training Readiness".into(), r });
    }

    // Sleep hours -> body battery high
    {
        let mut sleep_hrs = Vec::new();
        let mut bb_high = Vec::new();
        for d in daily {
            if let (Some(sd), Some(bb)) = (d.sleep_duration_secs, d.body_battery_high) {
                sleep_hrs.push(sd as f64 / 3600.0);
                bb_high.push(bb as f64);
            }
        }
        let r = pearson(&sleep_hrs, &bb_high);
        results.push(CorrelationPair { label: "Sleep Hours -> Body Battery High".into(), r });
    }

    // Steps -> stress
    {
        let mut steps = Vec::new();
        let mut stress = Vec::new();
        for d in daily {
            if let (Some(s), Some(st)) = (d.steps, d.avg_stress) {
                steps.push(s as f64);
                stress.push(st as f64);
            }
        }
        let r = pearson(&steps, &stress);
        results.push(CorrelationPair { label: "Steps -> Stress".into(), r });
    }

    results
}

fn bar_data(data: &[DailyData], extract: fn(&DailyData) -> Option<f64>) -> Vec<BarPoint> {
    data.iter().map(|d| BarPoint { label: d.date.clone(), value: extract(d).unwrap_or(0.0), color: None }).collect()
}

fn series_data(data: &[DailyData], label: &str, extract: fn(&DailyData) -> Option<f64>, color: &str) -> Series {
    Series {
        label: label.to_string(),
        points: data.iter().enumerate().filter_map(|(i, d)| extract(d).map(|v| (i as f64, v))).collect(),
        color: color.to_string(), fill: true,
    }
}

#[component]
pub fn TrendsPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    view! {
        <div class="animate-slide-up">
            <div class="flex flex-wrap items-center gap-3 mb-6">
                <div>
                    <h1 class="page-title">"Trends"</h1>
                    <p class="text-dim text-sm">"Long-term health analysis"</p>
                </div>
                <div class="flex gap-1 ml-auto">
                    {[7, 14, 30, 90, 180, 365].into_iter().map(|d| {
                        let label = if d == 365 { "1y".to_string() } else { format!("{d}d") };
                        view! {
                            <button
                                class=move || format!("px-3 py-2 text-xs rounded-lg font-display font-medium min-h-[36px] transition-all {}", if state.days.get() == d { "bg-accent text-bg" } else { "bg-white/[0.06] text-text hover:bg-white/[0.1]" })
                                on:click=move |_| {
                                    let s = state.clone();
                                    s.days.set(d);
                                    leptos::task::spawn_local(async move { s.load_daily().await; });
                                }
                            >{label}</button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            {move || {
                let d = state.daily_data.get();
                if d.is_empty() { return view! { <div class="card text-dim">"Loading..."</div> }.into_any(); }
                view! {
                    // === Activity ===
                    <h2 class="text-sm font-display font-semibold text-text mb-3">"Activity"</h2>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3 mb-3">
                        <BarChart title="Daily Steps".into() data=bar_data(&d, |d| d.steps.map(|v| v as f64))
                            color=theme::CHART_GREEN.into() thresholds=vec![(10000.0, theme::GOOD.into())] unit="steps".into() />
                        <BarChart title="Daily Distance".into() data=bar_data(&d, |d| d.distance_meters.map(|v| v / 1000.0))
                            color=theme::CHART_BLUE.into() thresholds=vec![(8.0, theme::GOOD.into())] unit="km".into() />
                        <BarChart title="Daily Calories".into() data=bar_data(&d, |d| d.total_calories.map(|v| v as f64))
                            color=theme::CHART_ORANGE.into() unit="kcal".into() />
                    </div>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3 mb-3">
                        <BarChart title="Active Calories".into() data=bar_data(&d, |d| d.active_calories.map(|v| v as f64))
                            color=theme::CHART_RED.into() unit="kcal".into() />
                        <BarChart title="Floors Climbed".into() data=bar_data(&d, |d| d.floors_climbed.map(|v| v as f64))
                            color=theme::CHART_PURPLE.into() unit="floors".into() />
                        <BarChart title="Intensity Minutes".into() data=bar_data(&d, |d| d.intensity_minutes.map(|v| v as f64))
                            color=theme::CHART_ORANGE.into() unit="min".into() />
                    </div>

                    // === Cardiovascular ===
                    <h2 class="text-sm font-display font-semibold text-text mt-6 mb-3">"Cardiovascular"</h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mb-3">
                        <TimeseriesChart title="RHR & HRV".into()
                            series=vec![
                                series_data(&d, "RHR", |d| d.resting_heart_rate.map(|v| v as f64), theme::CHART_RED),
                                series_data(&d, "HRV", |d| d.hrv_last_night, theme::CHART_GREEN),
                            ] />
                        <TimeseriesChart title="Stress & SpO2".into()
                            series=vec![
                                series_data(&d, "Stress", |d| d.avg_stress.map(|v| v as f64), theme::CHART_RED),
                                series_data(&d, "SpO2", |d| d.avg_spo2, theme::CHART_PURPLE),
                            ] />
                    </div>

                    // === Recovery ===
                    <h2 class="text-sm font-display font-semibold text-text mt-6 mb-3">"Recovery"</h2>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3 mb-3">
                        {
                            let bb_stacked: Vec<StackedBarPoint> = d.iter().map(|day| StackedBarPoint {
                                label: day.date.clone(),
                                segments: vec![
                                    (day.body_battery_charge.unwrap_or(0) as f64, theme::BB_CHARGED.into()),
                                    (day.body_battery_drain.unwrap_or(0).abs() as f64, theme::BB_DRAINED.into()),
                                ],
                            }).collect();
                            let bb_range = bar_data(&d, |d| {
                                let h = d.body_battery_high.unwrap_or(0) as f64;
                                let l = d.body_battery_low.unwrap_or(0) as f64;
                                Some(h - l)
                            });
                            view! {
                                <StackedBarChart title="Body Battery Charge/Drain".into() data=bb_stacked
                                    legend=vec![("Charged".into(), theme::BB_CHARGED.into()), ("Drained".into(), theme::BB_DRAINED.into())] />
                                <BarChart title="Body Battery Range".into() data=bb_range color=theme::CHART_PURPLE.into() />
                                <BarChart title="Sleep Score".into() data=bar_data(&d, |d| d.sleep_score.map(|v| v as f64))
                                    color=theme::CHART_BLUE.into() thresholds=vec![(80.0, theme::GOOD.into())] />
                            }
                        }
                    </div>

                    // === Body & Vitals ===
                    <h2 class="text-sm font-display font-semibold text-text mt-6 mb-3">"Body & Vitals"</h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mb-3">
                        <TimeseriesChart title="Stress (Avg & Max)".into()
                            series=vec![
                                series_data(&d, "Avg Stress", |d| d.avg_stress.map(|v| v as f64), theme::CHART_RED),
                                series_data(&d, "Max Stress", |d| d.max_stress.map(|v| v as f64), theme::CHART_ORANGE),
                            ] />
                        <TimeseriesChart title="Weight & BMI".into()
                            series=vec![
                                series_data(&d, "Weight", |d| d.weight_grams.map(|v| v / 1000.0), theme::CHART_YELLOW),
                                series_data(&d, "BMI", |d| d.bmi, theme::CHART_ORANGE),
                            ]
                            unit="kg".into() />
                    </div>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mb-3">
                        <TimeseriesChart title="Respiration & SpO2".into()
                            series=vec![
                                series_data(&d, "Respiration", |d| d.avg_respiration, theme::CHART_BLUE),
                                series_data(&d, "Lowest SpO2", |d| d.lowest_spo2, theme::CHART_PURPLE),
                            ] />
                        <TimeseriesChart title="Overnight HR & Restless".into()
                            series=vec![
                                series_data(&d, "Overnight HR", |d| d.sleep_avg_overnight_hr, theme::CHART_RED),
                                series_data(&d, "Restless", |d| d.sleep_restless_moments.map(|v| v as f64), theme::CHART_ORANGE),
                            ] />
                    </div>

                    // Body Composition
                    {
                        let has_body_data = d.iter().any(|d| d.body_fat_pct.is_some() || d.muscle_mass_grams.is_some());
                        if has_body_data {
                            let mut body_series = vec![
                                series_data(&d, "Body Fat %", |d| d.body_fat_pct, theme::CHART_ORANGE),
                            ];
                            if d.iter().any(|d| d.muscle_mass_grams.is_some()) {
                                body_series.push(series_data(&d, "Muscle kg", |d| d.muscle_mass_grams.map(|v| v / 1000.0), theme::CHART_GREEN));
                            }
                            view! {
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mb-3">
                                    <TimeseriesChart title="Body Composition".into()
                                        series=body_series />
                                    <TimeseriesChart title="Weight + Body Fat".into()
                                        series=vec![
                                            series_data(&d, "Weight kg", |d| d.weight_grams.map(|v| v / 1000.0), theme::CHART_YELLOW),
                                            series_data(&d, "Body Fat %", |d| d.body_fat_pct, theme::CHART_ORANGE),
                                        ] />
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }

                    // Fitness Age & Race Predictors from extended data
                    {
                        let ext = state.extended_data.get();
                        let has_fitness = ext.iter().any(|e| e.fitness_age.is_some());
                        let has_race = ext.iter().any(|e| e.race_5k_secs.is_some());
                        if has_fitness || has_race {
                            let fmt_race = |secs: f64| -> f64 { secs / 60.0 }; // show in minutes
                            view! {
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mb-3">
                                    {if has_fitness {
                                        let pts: Vec<(f64, f64)> = ext.iter().enumerate()
                                            .filter_map(|(i, e)| e.fitness_age.map(|v| (i as f64, v as f64))).collect();
                                        view! {
                                            <TimeseriesChart title="Fitness Age".into()
                                                series=vec![Series { label: "Fitness Age".into(), points: pts, color: theme::CHART_GREEN.into(), fill: true }]
                                                unit="years".into() />
                                        }.into_any()
                                    } else { view! { <div></div> }.into_any() }}
                                    {if has_race {
                                        view! {
                                            <TimeseriesChart title="Race Predictors".into()
                                                series=vec![
                                                    Series { label: "5K".into(), points: ext.iter().enumerate().filter_map(|(i, e)| e.race_5k_secs.map(|v| (i as f64, fmt_race(v)))).collect(), color: theme::CHART_GREEN.into(), fill: false },
                                                    Series { label: "10K".into(), points: ext.iter().enumerate().filter_map(|(i, e)| e.race_10k_secs.map(|v| (i as f64, fmt_race(v)))).collect(), color: theme::CHART_BLUE.into(), fill: false },
                                                    Series { label: "Half".into(), points: ext.iter().enumerate().filter_map(|(i, e)| e.race_half_secs.map(|v| (i as f64, fmt_race(v)))).collect(), color: theme::CHART_ORANGE.into(), fill: false },
                                                    Series { label: "Marathon".into(), points: ext.iter().enumerate().filter_map(|(i, e)| e.race_marathon_secs.map(|v| (i as f64, fmt_race(v)))).collect(), color: theme::CHART_RED.into(), fill: false },
                                                ]
                                                unit="min".into() />
                                        }.into_any()
                                    } else { view! { <div></div> }.into_any() }}
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }

                    // === Breakdowns ===
                    <h2 class="text-sm font-display font-semibold text-text mt-6 mb-3">"Breakdowns"</h2>
                    {
                        let ext = state.extended_data.get();
                        if ext.is_empty() {
                            view! { <div></div> }.into_any()
                        } else {
                            let stress_stacked: Vec<StackedBarPoint> = ext.iter().map(|day| StackedBarPoint {
                                label: day.date.clone().unwrap_or_default(),
                                segments: vec![
                                    (day.rest_stress_secs.unwrap_or(0) as f64, theme::STRESS_REST.into()),
                                    (day.low_stress_secs.unwrap_or(0) as f64, theme::STRESS_LOW.into()),
                                    (day.medium_stress_secs.unwrap_or(0) as f64, theme::STRESS_MEDIUM.into()),
                                    (day.high_stress_secs.unwrap_or(0) as f64, theme::STRESS_HIGH.into()),
                                ],
                            }).collect();

                            let activity_stacked: Vec<StackedBarPoint> = ext.iter().map(|day| StackedBarPoint {
                                label: day.date.clone().unwrap_or_default(),
                                segments: vec![
                                    (day.sedentary_secs.unwrap_or(0) as f64, theme::ACTIVITY_SEDENTARY.into()),
                                    (day.active_secs.unwrap_or(0) as f64, theme::ACTIVITY_ACTIVE.into()),
                                    (day.highly_active_secs.unwrap_or(0) as f64, theme::ACTIVITY_INTENSE.into()),
                                ],
                            }).collect();

                            view! {
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                    <StackedBarChart title="Stress Breakdown".into() data=stress_stacked
                                        legend=vec![
                                            ("Rest".into(), theme::STRESS_REST.into()),
                                            ("Low".into(), theme::STRESS_LOW.into()),
                                            ("Medium".into(), theme::STRESS_MEDIUM.into()),
                                            ("High".into(), theme::STRESS_HIGH.into()),
                                        ] />
                                    <StackedBarChart title="Activity Levels".into() data=activity_stacked
                                        legend=vec![
                                            ("Sedentary".into(), theme::ACTIVITY_SEDENTARY.into()),
                                            ("Active".into(), theme::ACTIVITY_ACTIVE.into()),
                                            ("Highly Active".into(), theme::ACTIVITY_INTENSE.into()),
                                        ] />
                                </div>
                            }.into_any()
                        }
                    }

                    // Correlations
                    {
                        let corrs = compute_correlations(&d);
                        let has_data = corrs.iter().any(|c| c.r.abs() > 0.0);
                        if has_data {
                            view! {
                                <div class="card mt-3">
                                    <div class="text-text text-sm font-display font-semibold mb-3">"Metric Correlations"</div>
                                    <div class="text-dim text-xs mb-3">"Pearson correlation between key health metrics"</div>
                                    <div class="space-y-2">
                                        {corrs.into_iter().map(|c| {
                                            let (strength_label, color) = correlation_strength(c.r);
                                            let bar_width = format!("{}%", (c.r.abs() * 100.0).min(100.0));
                                            let bar_direction = if c.r >= 0.0 { "right" } else { "left" };
                                            view! {
                                                <div class="flex items-center gap-3">
                                                    <div class="text-xs text-text w-56 flex-shrink-0">{c.label}</div>
                                                    <div class="flex-1 h-2 rounded-full relative" style=format!("background: {}22", theme::DIM)>
                                                        <div class="h-full rounded-full absolute" style=format!("width: {}; background: {}; {}: 50%;", bar_width, color, bar_direction)></div>
                                                    </div>
                                                    <div class="text-xs w-32 text-right flex-shrink-0" style=format!("color: {}", color)>
                                                        {format!("{:.2}", c.r)} " (" {strength_label} ")"
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }
                }.into_any()
            }}
        </div>
    }
}
