use leptos::prelude::*;
use crate::models::DailyData;
use crate::components::charts::bar_chart::*;
use crate::components::charts::timeseries::*;
use crate::components::charts::pie_chart::*;
use crate::components::charts::state_timeline::*;
use crate::theme;

fn bar_data(data: &[DailyData], extract: fn(&DailyData) -> Option<f64>) -> Vec<BarPoint> {
    data.iter().map(|d| BarPoint {
        label: d.date.clone(),
        value: extract(d).unwrap_or(0.0),
        color: None,
    }).collect()
}

fn series_data(data: &[DailyData], label: &str, extract: fn(&DailyData) -> Option<f64>, color: &str) -> Series {
    Series {
        label: label.to_string(),
        points: data.iter().enumerate().filter_map(|(i, d)| extract(d).map(|v| (i as f64, v))).collect(),
        color: color.to_string(),
        fill: true,
    }
}

#[component]
pub fn TrendsSection(data: ReadSignal<Vec<DailyData>>) -> impl IntoView {
    view! {
        <div>
            // Row 1: Steps, Distance, Calories
            <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                {move || {
                    let d = data.get();
                    view! {
                        <BarChart title="Daily Steps".into() data=bar_data(&d, |d| d.steps.map(|v| v as f64))
                            color=theme::CHART_GREEN.into()
                            thresholds=vec![(10000.0, theme::GOOD.into())]
                            unit="steps".into() />
                        <BarChart title="Daily Distance".into() data=bar_data(&d, |d| d.distance_meters.map(|v| v / 1000.0))
                            color=theme::CHART_BLUE.into()
                            thresholds=vec![(8.0, theme::GOOD.into())]
                            unit="km".into() />
                        <BarChart title="Daily Calories".into() data=bar_data(&d, |d| d.total_calories.map(|v| v as f64))
                            color=theme::CHART_ORANGE.into() unit="kcal".into() />
                    }
                }}
            </div>

            // Row 2: Health metrics timeseries
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                {move || {
                    let d = data.get();
                    view! {
                        <TimeseriesChart title="Daily Health Metrics".into()
                            series=vec![
                                series_data(&d, "RHR", |d| d.resting_heart_rate.map(|v| v as f64), theme::CHART_RED),
                                series_data(&d, "HRV", |d| d.hrv_last_night, theme::CHART_GREEN),
                                series_data(&d, "SpO2", |d| d.avg_spo2, theme::CHART_PURPLE),
                            ]
                            unit="".into() />
                        <TimeseriesChart title="Stress & Recovery".into()
                            series=vec![
                                series_data(&d, "Stress", |d| d.avg_stress.map(|v| v as f64), theme::CHART_RED),
                                series_data(&d, "Respiration", |d| d.avg_respiration, theme::CHART_PURPLE),
                            ]
                            unit="".into() />
                    }
                }}
            </div>

            // Row 3: VO2 Max and Training
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                {move || {
                    let d = data.get();
                    view! {
                        <TimeseriesChart title="VO2 Max".into()
                            series=vec![series_data(&d, "VO2", |d| d.vo2_max, theme::CHART_GREEN)]
                            unit="ml/kg/min".into() />
                        <TimeseriesChart title="Training Load & Readiness".into()
                            series=vec![
                                series_data(&d, "Load", |d| d.training_load, theme::CHART_ORANGE),
                                series_data(&d, "Readiness", |d| d.training_readiness, theme::CHART_BLUE),
                            ]
                            unit="".into() />
                    }
                }}
            </div>

            // Row 4: Sleep
            <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                {move || {
                    let d = data.get();
                    // Sleep stages stacked
                    let sleep_stacked: Vec<StackedBarPoint> = d.iter().map(|day| StackedBarPoint {
                        label: day.date.clone(),
                        segments: vec![
                            (day.deep_sleep_secs.unwrap_or(0) as f64, theme::SLEEP_DEEP.into()),
                            (day.light_sleep_secs.unwrap_or(0) as f64, theme::SLEEP_LIGHT.into()),
                            (day.rem_sleep_secs.unwrap_or(0) as f64, theme::SLEEP_REM.into()),
                            (day.awake_secs.unwrap_or(0) as f64, theme::SLEEP_AWAKE.into()),
                        ],
                    }).collect();
                    // Sleep score
                    let sleep_score_data = bar_data(&d, |d| d.sleep_score.map(|v| v as f64));
                    // Average sleep pie
                    let total_deep: f64 = d.iter().filter_map(|d| d.deep_sleep_secs).sum::<i64>() as f64;
                    let total_light: f64 = d.iter().filter_map(|d| d.light_sleep_secs).sum::<i64>() as f64;
                    let total_rem: f64 = d.iter().filter_map(|d| d.rem_sleep_secs).sum::<i64>() as f64;
                    let total_awake: f64 = d.iter().filter_map(|d| d.awake_secs).sum::<i64>() as f64;
                    view! {
                        <StackedBarChart title="Sleep Stages".into() data=sleep_stacked unit="".into()
                            legend=vec![
                                ("Deep".into(), theme::SLEEP_DEEP.into()),
                                ("Light".into(), theme::SLEEP_LIGHT.into()),
                                ("REM".into(), theme::SLEEP_REM.into()),
                                ("Awake".into(), theme::SLEEP_AWAKE.into()),
                            ] />
                        <BarChart title="Sleep Score".into() data=sleep_score_data
                            color=theme::CHART_BLUE.into()
                            thresholds=vec![(80.0, theme::GOOD.into())] />
                        <PieChart title="Avg Sleep Breakdown".into()
                            slices=vec![
                                PieSlice { label: "Deep".into(), value: total_deep, color: theme::SLEEP_DEEP.into() },
                                PieSlice { label: "Light".into(), value: total_light, color: theme::SLEEP_LIGHT.into() },
                                PieSlice { label: "REM".into(), value: total_rem, color: theme::SLEEP_REM.into() },
                                PieSlice { label: "Awake".into(), value: total_awake, color: theme::SLEEP_AWAKE.into() },
                            ]
                            format_fn=theme::fmt_hours />
                    }
                }}
            </div>

            // Row 5: Body Battery
            <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                {move || {
                    let d = data.get();
                    let bb_stacked: Vec<StackedBarPoint> = d.iter().map(|day| StackedBarPoint {
                        label: day.date.clone(),
                        segments: vec![
                            (day.body_battery_charge.unwrap_or(0) as f64, theme::BB_CHARGED.into()),
                            (day.body_battery_drain.unwrap_or(0).abs() as f64, theme::BB_DRAINED.into()),
                        ],
                    }).collect();
                    let bb_range = bar_data(&d, |d| {
                        let high = d.body_battery_high.unwrap_or(0) as f64;
                        let low = d.body_battery_low.unwrap_or(0) as f64;
                        Some(high - low)
                    });
                    let intensity_stacked: Vec<StackedBarPoint> = d.iter().map(|day| {
                        let im = day.intensity_minutes.unwrap_or(0) as f64;
                        StackedBarPoint {
                            label: day.date.clone(),
                            segments: vec![(im, theme::CHART_ORANGE.into())],
                        }
                    }).collect();
                    view! {
                        <StackedBarChart title="Body Battery Charge/Drain".into() data=bb_stacked unit="".into()
                            legend=vec![("Charged".into(), theme::BB_CHARGED.into()), ("Drained".into(), theme::BB_DRAINED.into())] />
                        <BarChart title="Body Battery Range (High-Low)".into() data=bb_range
                            color=theme::CHART_PURPLE.into() />
                        <StackedBarChart title="Intensity Minutes".into() data=intensity_stacked unit="min".into()
                            legend=vec![("Intensity".into(), theme::CHART_ORANGE.into())] />
                    }
                }}
            </div>

            // Row 6: Stress, HR Range, Weight
            <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                {move || {
                    let d = data.get();
                    let stress_data = bar_data(&d, |d| d.avg_stress.map(|v| v as f64));
                    let hr_range: Vec<StackedBarPoint> = d.iter().map(|day| StackedBarPoint {
                        label: day.date.clone(),
                        segments: vec![
                            (day.min_heart_rate.unwrap_or(0) as f64, "#181b1f".into()),
                            ((day.max_heart_rate.unwrap_or(0) - day.min_heart_rate.unwrap_or(0)) as f64, theme::CHART_YELLOW.into()),
                        ],
                    }).collect();
                    view! {
                        <BarChart title="Daily Stress".into() data=stress_data
                            color=theme::CHART_RED.into()
                            thresholds=vec![(50.0, theme::STRESS_MEDIUM.into()), (75.0, theme::STRESS_HIGH.into())] />
                        <StackedBarChart title="Heart Rate Range (Min-Max)".into() data=hr_range
                            legend=vec![("Min".into(), "#181b1f".into()), ("Range".into(), theme::CHART_YELLOW.into())] />
                        <TimeseriesChart title="Weight".into()
                            series=vec![series_data(&d, "Weight", |d| d.weight_grams.map(|v| v / 1000.0), theme::CHART_YELLOW)]
                            unit="kg".into() />
                    }
                }}
            </div>

            // Row 7: Month at a Glance - multi-metric state timeline
            {move || {
                let d = data.get();
                if d.len() < 7 { return view! { <div></div> }.into_any(); }
                let make_row = |label: &str, extract: fn(&DailyData) -> Option<f64>, low_color: &str, high_color: &str| -> TimelineRow {
                    let vals: Vec<f64> = d.iter().filter_map(|d| extract(d)).collect();
                    let max = vals.iter().cloned().fold(f64::MIN, f64::max).max(0.001);
                    TimelineRow {
                        label: label.to_string(),
                        segments: d.iter().map(|day| {
                            let v = extract(day).unwrap_or(0.0);
                            let t = (v / max).clamp(0.0, 1.0);
                            TimelineSegment {
                                label: day.date.clone(),
                                value: 1.0,
                                color: theme::lerp_color(low_color, high_color, t),
                            }
                        }).collect(),
                    }
                };
                let rows = vec![
                    make_row("Steps", |d| d.steps.map(|v| v as f64), theme::BG, theme::CHART_GREEN),
                    make_row("RHR", |d| d.resting_heart_rate.map(|v| v as f64), theme::CHART_GREEN, theme::CHART_RED),
                    make_row("HRV", |d| d.hrv_last_night, theme::BG, theme::CHART_GREEN),
                    make_row("Sleep", |d| d.sleep_score.map(|v| v as f64), theme::CHART_RED, theme::CHART_BLUE),
                    make_row("Stress", |d| d.avg_stress.map(|v| v as f64), theme::CHART_GREEN, theme::CHART_RED),
                    make_row("Battery", |d| d.body_battery_high.map(|v| v as f64), theme::CHART_RED, theme::CHART_GREEN),
                    make_row("SpO2", |d| d.avg_spo2, theme::CHART_RED, theme::CHART_BLUE),
                    make_row("Readiness", |d| d.training_readiness, theme::BG, theme::CHART_GREEN),
                ];
                view! { <StateTimeline title="Month at a Glance".into() rows=rows /> }.into_any()
            }}
        </div>
    }
}
