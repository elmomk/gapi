use leptos::prelude::*;
use crate::state::AppState;
use crate::theme;
use crate::components::charts::timeseries::*;
use crate::components::charts::bar_chart::*;
use crate::components::charts::pie_chart::*;
use crate::components::charts::gauge::Gauge;
use crate::models::DailyData;

#[component]
pub fn SleepPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    view! {
        <div class="animate-slide-up">
            <h1 class="page-title">"Sleep"</h1>
            <p class="page-subtitle">"Sleep quality and patterns"</p>

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

                let total_deep: f64 = d.iter().filter_map(|d| d.deep_sleep_secs).sum::<i64>() as f64;
                let total_light: f64 = d.iter().filter_map(|d| d.light_sleep_secs).sum::<i64>() as f64;
                let total_rem: f64 = d.iter().filter_map(|d| d.rem_sleep_secs).sum::<i64>() as f64;
                let total_awake: f64 = d.iter().filter_map(|d| d.awake_secs).sum::<i64>() as f64;

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
