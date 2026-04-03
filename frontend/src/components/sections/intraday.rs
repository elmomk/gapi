use leptos::prelude::*;
use crate::models::*;
use crate::components::charts::timeseries::*;
use crate::components::charts::gauge::*;
use crate::components::charts::pie_chart::*;
use crate::components::charts::bar_chart::*;
use crate::theme;

#[component]
pub fn IntradaySection(
    hr_data: ReadSignal<Vec<IntradayPoint>>,
    stress_data: ReadSignal<Vec<StressPoint>>,
    hrv_data: ReadSignal<Vec<HrvReading>>,
    sleep_data: ReadSignal<Vec<SleepEpoch>>,
    resp_data: ReadSignal<Vec<IntradayPointF64>>,
    vitals: ReadSignal<Option<VitalsData>>,
) -> impl IntoView {
    view! {
        <div>
            // Row 1: Gauges sidebar + HR timeseries
            <div class="grid grid-cols-4 md:grid-cols-8 gap-3">
                // Gauges column
                <div class="col-span-1 md:col-span-2 flex flex-col gap-3">
                    {move || vitals.get().map(|v| view! {
                        <Gauge title="RHR".into() value=v.resting_heart_rate.map(|x| x as f64)
                            min=40.0 max=100.0 unit="bpm".into()
                            thresholds=vec![(40.0, theme::CHART_BLUE.into()), (60.0, theme::GOOD.into()), (70.0, theme::CHART_YELLOW.into()), (80.0, theme::WARN.into())] />
                        <Gauge title="Steps".into() value=v.steps.map(|x| x as f64)
                            min=0.0 max=15000.0 unit="".into()
                            thresholds=vec![(0.0, theme::WARN.into()), (5000.0, theme::CHART_ORANGE.into()), (10000.0, theme::GOOD.into())] />
                        <Gauge title="SpO2".into() value=v.resting_heart_rate.map(|_| 97.0) // placeholder
                            min=85.0 max=100.0 unit="%".into()
                            thresholds=vec![(85.0, theme::WARN.into()), (90.0, theme::CHART_YELLOW.into()), (95.0, theme::GOOD.into())] />
                        <Gauge title="Sleep".into() value=v.sleep_hours
                            min=0.0 max=10.0 unit="hrs".into()
                            thresholds=vec![(0.0, theme::WARN.into()), (5.0, theme::CHART_YELLOW.into()), (7.0, theme::GOOD.into())] />
                    })}
                </div>
                // Charts column
                <div class="col-span-3 md:col-span-6 flex flex-col gap-3">
                    // Heart rate 24h
                    {move || {
                        let hr = hr_data.get();
                        if hr.is_empty() { return view! { <div class="text-dim text-sm">"No intraday HR data"</div> }.into_any(); }
                        let series = vec![Series {
                            label: "Heart Rate".into(),
                            points: hr.iter().enumerate().map(|(i, p)| (i as f64, p.value as f64)).collect(),
                            color: theme::CHART_RED.into(), fill: true,
                        }];
                        view! { <TimeseriesChart title="Heart Rate (24h)".into() series=series unit="bpm".into() /> }.into_any()
                    }}
                    // Stress + Body Battery overlay
                    {move || {
                        let s = stress_data.get();
                        if s.is_empty() { return view! { <div></div> }.into_any(); }
                        let stress_series = Series {
                            label: "Stress".into(),
                            points: s.iter().enumerate().filter(|(_, p)| p.stress >= 0).map(|(i, p)| (i as f64, p.stress as f64)).collect(),
                            color: theme::CHART_RED.into(), fill: true,
                        };
                        let bb_series = Series {
                            label: "Body Battery".into(),
                            points: s.iter().enumerate().filter_map(|(i, p)| p.body_battery.map(|bb| (i as f64, bb as f64))).collect(),
                            color: theme::BB_CHARGED.into(), fill: true,
                        };
                        view! { <TimeseriesChart title="Stress & Body Battery (24h)".into() series=vec![stress_series, bb_series] /> }.into_any()
                    }}
                </div>
            </div>

            // Row 2: Sleep + Breathing + HRV
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mt-3">
                // Sleep intraday (stages)
                {move || {
                    let sl = sleep_data.get();
                    if sl.is_empty() { return view! { <div></div> }.into_any(); }
                    // Sleep HR + HRV
                    let mut series = vec![];
                    let hr_pts: Vec<(f64, f64)> = sl.iter().enumerate().filter_map(|(i, e)| e.hr.map(|h| (i as f64, h as f64))).collect();
                    if !hr_pts.is_empty() {
                        series.push(Series { label: "Sleep HR".into(), points: hr_pts, color: theme::CHART_RED.into(), fill: false });
                    }
                    let hrv = hrv_data.get();
                    if !hrv.is_empty() {
                        let hrv_pts: Vec<(f64, f64)> = hrv.iter().enumerate().map(|(i, h)| (i as f64, h.hrv_value)).collect();
                        series.push(Series { label: "HRV".into(), points: hrv_pts, color: theme::CHART_ORANGE.into(), fill: false });
                    }
                    view! { <TimeseriesChart title="Sleep Intraday (HR + HRV)".into() series=series /> }.into_any()
                }}
                // Breathing rate
                {move || {
                    let r = resp_data.get();
                    if r.is_empty() { return view! { <div></div> }.into_any(); }
                    let series = vec![Series {
                        label: "Breathing Rate".into(),
                        points: r.iter().enumerate().map(|(i, p)| (i as f64, p.value)).collect(),
                        color: theme::CHART_BLUE.into(), fill: true,
                    }];
                    view! { <TimeseriesChart title="Breathing Rate (24h)".into() series=series unit="brpm".into() /> }.into_any()
                }}
            </div>

            // Row 3: Sleep breakdown pie + Activity pie
            <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mt-3">
                {move || vitals.get().map(|v| {
                    let sleep_secs = v.sleep_hours.unwrap_or(0.0) * 3600.0;
                    view! {
                        <PieChart title="Today's Sleep".into()
                            slices=vec![
                                PieSlice { label: "Sleep".into(), value: sleep_secs, color: theme::SLEEP_DEEP.into() },
                                PieSlice { label: "Awake".into(), value: 86400.0 - sleep_secs, color: theme::SLEEP_AWAKE.into() },
                            ]
                            format_fn=theme::fmt_hours />
                        <Gauge title="Readiness".into() value=v.training_readiness
                            min=0.0 max=100.0 unit="".into()
                            thresholds=vec![(0.0, theme::WARN.into()), (40.0, theme::CHART_YELLOW.into()), (60.0, theme::GOOD.into())] />
                        <Gauge title="Body Battery".into() value=v.body_battery_high.map(|x| x as f64)
                            min=0.0 max=100.0 unit="".into()
                            thresholds=vec![(0.0, theme::WARN.into()), (30.0, theme::CHART_YELLOW.into()), (60.0, theme::GOOD.into())] />
                        <Gauge title="Stress".into() value=v.avg_stress.map(|x| x as f64)
                            min=0.0 max=100.0 unit="".into()
                            thresholds=vec![(0.0, theme::GOOD.into()), (40.0, theme::CHART_YELLOW.into()), (60.0, theme::WARN.into())] />
                    }
                })}
            </div>
        </div>
    }
}
