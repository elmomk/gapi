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

#[component]
pub fn HeartPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    view! {
        <div class="animate-slide-up">
            <h1 class="page-title">"Heart & Body"</h1>
            <p class="page-subtitle">"Cardiovascular and body metrics"</p>

            // Gauges
            {move || state.vitals.get().map(|v| view! {
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
                    <Gauge title="RHR".into() value=v.resting_heart_rate.map(|x| x as f64)
                        min=40.0 max=100.0 unit="bpm".into()
                        thresholds=vec![(40.0, theme::CHART_BLUE.into()), (60.0, theme::GOOD.into()), (70.0, theme::CHART_YELLOW.into()), (80.0, theme::WARN.into())] />
                    <Gauge title="SpO2".into() value=v.resting_heart_rate.map(|_| 97.0)
                        min=85.0 max=100.0 unit="%".into()
                        thresholds=vec![(85.0, theme::WARN.into()), (90.0, theme::CHART_YELLOW.into()), (95.0, theme::GOOD.into())] />
                    <Gauge title="Body Battery".into() value=v.body_battery_high.map(|x| x as f64)
                        min=0.0 max=100.0 unit="%".into()
                        thresholds=vec![(0.0, theme::WARN.into()), (30.0, theme::CHART_YELLOW.into()), (60.0, theme::GOOD.into())] />
                    <Gauge title="Stress".into() value=v.avg_stress.map(|x| x as f64)
                        min=0.0 max=100.0 unit="".into()
                        thresholds=vec![(0.0, theme::GOOD.into()), (40.0, theme::CHART_YELLOW.into()), (60.0, theme::WARN.into())] />
                </div>
            })}

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
                    <TimeseriesChart title="Weight".into()
                        series=vec![Series { label: "Weight".into(), points: d.iter().enumerate().filter_map(|(i, d)| d.weight_grams.map(|v| (i as f64, v / 1000.0))).collect(), color: theme::CHART_YELLOW.into(), fill: true }]
                        unit="kg".into() />
                }.into_any()
            }}
        </div>
    }
}
