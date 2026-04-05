use leptos::prelude::*;
use crate::state::AppState;
use crate::theme;
use crate::components::charts::timeseries::*;
use crate::components::charts::bar_chart::*;
use crate::components::charts::gauge::Gauge;
use crate::models::DailyData;

fn bar_data(data: &[DailyData], extract: fn(&DailyData) -> Option<f64>) -> Vec<BarPoint> {
    data.iter().map(|d| BarPoint { label: d.date.clone(), value: extract(d).unwrap_or(0.0), color: None }).collect()
}

fn hrv_trend(daily: &[DailyData]) -> (Option<f64>, Option<f64>, &'static str) {
    let recent_7: Vec<f64> = daily.iter().rev().take(7).filter_map(|d| d.hrv_last_night).collect();
    let recent_30: Vec<f64> = daily.iter().rev().take(30).filter_map(|d| d.hrv_last_night).collect();
    let avg_7 = if recent_7.is_empty() { None } else { Some(recent_7.iter().sum::<f64>() / recent_7.len() as f64) };
    let avg_30 = if recent_30.is_empty() { None } else { Some(recent_30.iter().sum::<f64>() / recent_30.len() as f64) };
    let trend = match (avg_7, avg_30) {
        (Some(a), Some(b)) if a > b * 1.05 => "Improving",
        (Some(a), Some(b)) if a < b * 0.95 => "Declining",
        _ => "Stable",
    };
    (avg_7, avg_30, trend)
}

#[component]
pub fn HeartPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    view! {
        <div class="animate-slide-up">
            <h1 class="page-title">"Heart & Body"</h1>
            <p class="page-subtitle">"Cardiovascular and body metrics"</p>

            // HRV Trend Analysis Card
            {move || {
                let daily = state.daily_data.get();
                let (avg_7, avg_30, trend) = hrv_trend(&daily);
                if avg_7.is_none() && avg_30.is_none() { return view! { <div></div> }.into_any(); }
                let (trend_color, trend_arrow) = match trend {
                    "Improving" => (theme::GOOD, "\u{2191}"),
                    "Declining" => (theme::WARN, "\u{2193}"),
                    _ => (theme::CHART_YELLOW, "\u{2194}"),
                };
                view! {
                    <div class="card mb-6" style=format!("border-left: 3px solid {}", trend_color)>
                        <div class="metric-label mb-1">"HRV Trend"</div>
                        <div class="flex items-center gap-4">
                            <div>
                                <span class="text-2xl font-display font-bold" style=format!("color: {}", trend_color)>
                                    {trend_arrow} " " {trend}
                                </span>
                            </div>
                            <div class="text-sm text-dim">
                                {avg_7.map(|v| format!("7-day avg: {:.0}ms", v)).unwrap_or_default()}
                                " / "
                                {avg_30.map(|v| format!("30-day avg: {:.0}ms", v)).unwrap_or_default()}
                            </div>
                        </div>
                    </div>
                }.into_any()
            }}

            // Gauges
            {move || state.vitals.get().map(|v| {
                let d = state.daily_data.get();
                let today = d.last();
                let spo2 = today.and_then(|d| d.avg_spo2);
                let lowest_spo2 = today.and_then(|d| d.lowest_spo2);
                let avg_hr = today.and_then(|d| d.avg_heart_rate);
                let hrv_weekly = today.and_then(|d| d.hrv_weekly_avg);
                let hrv_status = today.and_then(|d| d.hrv_status.clone());
                let avg_resp = today.and_then(|d| d.avg_respiration);
                view! {
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
                    <Gauge title="RHR".into() value=v.resting_heart_rate.map(|x| x as f64)
                        min=40.0 max=100.0 unit="bpm".into()
                        thresholds=vec![(40.0, theme::CHART_BLUE.into()), (60.0, theme::GOOD.into()), (70.0, theme::CHART_YELLOW.into()), (80.0, theme::WARN.into())] />
                    <Gauge title="Avg HR".into() value=avg_hr.map(|x| x as f64)
                        min=40.0 max=120.0 unit="bpm".into()
                        thresholds=vec![(40.0, theme::CHART_BLUE.into()), (70.0, theme::GOOD.into()), (85.0, theme::CHART_YELLOW.into()), (100.0, theme::WARN.into())] />
                    <Gauge title="SpO2".into() value=spo2
                        min=85.0 max=100.0 unit="%".into()
                        thresholds=vec![(85.0, theme::WARN.into()), (90.0, theme::CHART_YELLOW.into()), (95.0, theme::GOOD.into())] />
                    <Gauge title="Body Battery".into() value=v.body_battery_high.map(|x| x as f64)
                        min=0.0 max=100.0 unit="%".into()
                        thresholds=vec![(0.0, theme::WARN.into()), (30.0, theme::CHART_YELLOW.into()), (60.0, theme::GOOD.into())] />
                </div>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
                    <Gauge title="Stress".into() value=v.avg_stress.map(|x| x as f64)
                        min=0.0 max=100.0 unit="".into()
                        thresholds=vec![(0.0, theme::GOOD.into()), (40.0, theme::CHART_YELLOW.into()), (60.0, theme::WARN.into())] />
                    <Gauge title="Respiration".into() value=avg_resp
                        min=8.0 max=30.0 unit="brpm".into()
                        thresholds=vec![(8.0, theme::CHART_BLUE.into()), (12.0, theme::GOOD.into()), (20.0, theme::CHART_YELLOW.into()), (25.0, theme::WARN.into())] />
                    <Gauge title="Lowest SpO2".into() value=lowest_spo2
                        min=85.0 max=100.0 unit="%".into()
                        thresholds=vec![(85.0, theme::WARN.into()), (90.0, theme::CHART_YELLOW.into()), (95.0, theme::GOOD.into())] />
                    <Gauge title="HRV Weekly".into() value=hrv_weekly
                        min=0.0 max=150.0 unit="ms".into()
                        thresholds=vec![(0.0, theme::WARN.into()), (30.0, theme::CHART_YELLOW.into()), (50.0, theme::GOOD.into())] />
                </div>
                // HRV Status badge
                {hrv_status.map(|status| {
                    let color = match status.as_str() {
                        "BALANCED" | "HIGH" => theme::GOOD,
                        "UNBALANCED" | "LOW" => theme::WARN,
                        _ => theme::CHART_YELLOW,
                    };
                    view! {
                        <div class="card mb-6" style=format!("border-left: 3px solid {}", color)>
                            <div class="flex items-center gap-3">
                                <span class="metric-label">"HRV Status"</span>
                                <span class="text-sm font-display font-semibold" style=format!("color: {}", color)>{status}</span>
                            </div>
                        </div>
                    }
                })}
            }})}

            // 24h Heart Rate
            {move || {
                let hr = state.intraday_hr.get();
                if hr.is_empty() { return view! { <div class="card text-dim text-sm">"No intraday HR data for today"</div> }.into_any(); }
                let series = vec![Series {
                    label: "Heart Rate".into(),
                    points: hr.iter().enumerate().map(|(i, p)| (i as f64, p.value as f64)).collect(),
                    color: theme::CHART_RED.into(), fill: true,
                }];
                view! { <TimeseriesChart title="Heart Rate (24h)".into() series=series unit="bpm".into() /> }.into_any()
            }}

            // Stress + Body Battery
            {move || {
                let s = state.intraday_stress.get();
                if s.is_empty() { return view! { <div></div> }.into_any(); }
                let series = vec![
                    Series { label: "Stress".into(), points: s.iter().enumerate().filter(|(_, p)| p.stress >= 0).map(|(i, p)| (i as f64, p.stress as f64)).collect(), color: theme::CHART_RED.into(), fill: true },
                    Series { label: "Body Battery".into(), points: s.iter().enumerate().filter_map(|(i, p)| p.body_battery.map(|bb| (i as f64, bb as f64))).collect(), color: theme::BB_CHARGED.into(), fill: true },
                ];
                view! { <TimeseriesChart title="Stress & Body Battery (24h)".into() series=series /> }.into_any()
            }}

            // Breathing rate
            {move || {
                let r = state.intraday_resp.get();
                if r.is_empty() { return view! { <div></div> }.into_any(); }
                let series = vec![Series {
                    label: "Breathing Rate".into(),
                    points: r.iter().enumerate().map(|(i, p)| (i as f64, p.value)).collect(),
                    color: theme::CHART_BLUE.into(), fill: true,
                }];
                view! { <TimeseriesChart title="Breathing Rate (24h)".into() series=series unit="brpm".into() /> }.into_any()
            }}

            // Long-term trends
            <h2 class="text-sm font-display font-semibold text-text mt-8 mb-3">"Trends"</h2>
            {move || {
                let d = state.daily_data.get();
                if d.is_empty() { return view! { <div></div> }.into_any(); }
                let rhr_series = Series { label: "RHR".into(), points: d.iter().enumerate().filter_map(|(i, d)| d.resting_heart_rate.map(|v| (i as f64, v as f64))).collect(), color: theme::CHART_RED.into(), fill: true };
                let hrv_series = Series { label: "HRV".into(), points: d.iter().enumerate().filter_map(|(i, d)| d.hrv_last_night.map(|v| (i as f64, v))).collect(), color: theme::CHART_GREEN.into(), fill: true };
                let hr_range: Vec<StackedBarPoint> = d.iter().map(|day| StackedBarPoint {
                    label: day.date.clone(),
                    segments: vec![
                        (day.min_heart_rate.unwrap_or(0) as f64, "#181b1f".into()),
                        ((day.max_heart_rate.unwrap_or(0) - day.min_heart_rate.unwrap_or(0)) as f64, theme::CHART_YELLOW.into()),
                    ],
                }).collect();
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                        <TimeseriesChart title="RHR & HRV Trend".into() series=vec![rhr_series, hrv_series] />
                        <StackedBarChart title="Heart Rate Range (Min-Max)".into() data=hr_range legend=vec![("Min".into(), "#181b1f".into()), ("Range".into(), theme::CHART_YELLOW.into())] />
                    </div>
                    <TimeseriesChart title="Weight & BMI".into()
                        series=vec![
                            Series { label: "Weight".into(), points: d.iter().enumerate().filter_map(|(i, d)| d.weight_grams.map(|v| (i as f64, v / 1000.0))).collect(), color: theme::CHART_YELLOW.into(), fill: true },
                            Series { label: "BMI".into(), points: d.iter().enumerate().filter_map(|(i, d)| d.bmi.map(|v| (i as f64, v))).collect(), color: theme::CHART_ORANGE.into(), fill: false },
                        ]
                        unit="kg".into() />
                }.into_any()
            }}
        </div>
    }
}
