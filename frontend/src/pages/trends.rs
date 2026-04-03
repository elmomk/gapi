use leptos::prelude::*;
use crate::state::AppState;
use crate::theme;
use crate::components::charts::bar_chart::*;
use crate::components::charts::timeseries::*;
use crate::models::DailyData;

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
                    // Steps, Distance, Calories
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3 mb-3">
                        <BarChart title="Daily Steps".into() data=bar_data(&d, |d| d.steps.map(|v| v as f64))
                            color=theme::CHART_GREEN.into() thresholds=vec![(10000.0, theme::GOOD.into())] unit="steps".into() />
                        <BarChart title="Daily Distance".into() data=bar_data(&d, |d| d.distance_meters.map(|v| v / 1000.0))
                            color=theme::CHART_BLUE.into() thresholds=vec![(8.0, theme::GOOD.into())] unit="km".into() />
                        <BarChart title="Daily Calories".into() data=bar_data(&d, |d| d.total_calories.map(|v| v as f64))
                            color=theme::CHART_ORANGE.into() unit="kcal".into() />
                    </div>

                    // Health metrics
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

                    // VO2 + Training
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mb-3">
                        <TimeseriesChart title="VO2 Max".into()
                            series=vec![series_data(&d, "VO2", |d| d.vo2_max, theme::CHART_GREEN)]
                            unit="ml/kg/min".into() />
                        <TimeseriesChart title="Training Load & Readiness".into()
                            series=vec![
                                series_data(&d, "Load", |d| d.training_load, theme::CHART_ORANGE),
                                series_data(&d, "Readiness", |d| d.training_readiness, theme::CHART_BLUE),
                            ] />
                    </div>

                    // Body Battery
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
                            let intensity: Vec<BarPoint> = d.iter().map(|d| BarPoint {
                                label: d.date.clone(), value: d.intensity_minutes.unwrap_or(0) as f64, color: None,
                            }).collect();
                            view! {
                                <StackedBarChart title="Body Battery Charge/Drain".into() data=bb_stacked
                                    legend=vec![("Charged".into(), theme::BB_CHARGED.into()), ("Drained".into(), theme::BB_DRAINED.into())] />
                                <BarChart title="Body Battery Range".into() data=bb_range color=theme::CHART_PURPLE.into() />
                                <BarChart title="Intensity Minutes".into() data=intensity color=theme::CHART_ORANGE.into() unit="min".into() />
                            }
                        }
                    </div>

                    // Stress + Weight
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mb-3">
                        <BarChart title="Daily Stress".into() data=bar_data(&d, |d| d.avg_stress.map(|v| v as f64))
                            color=theme::CHART_RED.into()
                            thresholds=vec![(50.0, theme::STRESS_MEDIUM.into()), (75.0, theme::STRESS_HIGH.into())] />
                        <TimeseriesChart title="Weight".into()
                            series=vec![series_data(&d, "Weight", |d| d.weight_grams.map(|v| v / 1000.0), theme::CHART_YELLOW)]
                            unit="kg".into() />
                    </div>

                    // Stress breakdown + Activity levels from extended data
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
                }.into_any()
            }}
        </div>
    }
}
