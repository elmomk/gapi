use leptos::prelude::*;
use crate::state::AppState;
use crate::theme;
use crate::components::charts::timeseries::*;

fn fmt_race_time(secs: f64) -> String {
    let h = (secs / 3600.0) as i64;
    let m = ((secs % 3600.0) / 60.0) as i64;
    let s = (secs % 60.0) as i64;
    if h > 0 { format!("{}:{:02}:{:02}", h, m, s) }
    else { format!("{}:{:02}", m, s) }
}

#[component]
pub fn TrainingPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    view! {
        <div class="animate-slide-up">
            <h1 class="page-title">"Training"</h1>
            <p class="page-subtitle">"Performance and readiness"</p>

            // Training Readiness hero + Training Status Phase
            {move || {
                let d = state.daily_data.get();
                let ext = state.extended_data.get();
                let readiness = d.last().and_then(|d| d.training_readiness);
                let phase = ext.last().and_then(|e| e.training_status_phase.clone());
                if readiness.is_none() && phase.is_none() { return view! { <div></div> }.into_any(); }
                let score = readiness.unwrap_or(0.0);
                let score_color = if score >= 66.0 { theme::GOOD } else if score >= 33.0 { theme::CHART_YELLOW } else { theme::WARN };
                let phase_color = match phase.as_deref() {
                    Some("PRODUCTIVE") | Some("PEAKING") => theme::GOOD,
                    Some("RECOVERY") | Some("MAINTAINING") => theme::CHART_YELLOW,
                    Some("UNPRODUCTIVE") | Some("DETRAINING") | Some("OVERREACHING") => theme::WARN,
                    _ => theme::INFO,
                };
                view! {
                    <div class="card mb-6" style=format!("border-left: 3px solid {}", score_color)>
                        <div class="flex items-center justify-between flex-wrap gap-4">
                            <div>
                                <div class="metric-label mb-1">"Training Readiness"</div>
                                <div class="flex items-baseline gap-2">
                                    <span class="text-4xl font-display font-bold" style=format!("color: {}", score_color)>
                                        {format!("{:.0}", score)}
                                    </span>
                                    <span class="text-dim text-sm font-display">"/100"</span>
                                </div>
                            </div>
                            {phase.map(|p| view! {
                                <div class="px-4 py-2 rounded-lg" style=format!("background: {}22; border: 1px solid {}44", phase_color, phase_color)>
                                    <span class="text-sm font-display font-bold" style=format!("color: {}", phase_color)>{p}</span>
                                </div>
                            })}
                        </div>
                    </div>
                }.into_any()
            }}

            // Stats cards: VO2 Max, Fitness Age, Acute Load, Training Load
            {move || {
                let d = state.daily_data.get();
                let ext = state.extended_data.get();
                let vo2 = d.last().and_then(|d| d.vo2_max);
                let load = d.last().and_then(|d| d.training_load);
                let fitness_age = ext.last().and_then(|e| e.fitness_age);
                let acute = ext.last().and_then(|e| e.acute_training_load);
                if vo2.is_none() && fitness_age.is_none() && load.is_none() && acute.is_none() {
                    return view! { <div></div> }.into_any();
                }
                view! {
                    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
                        <div class="card" style=format!("border-left: 3px solid {}", theme::CHART_GREEN)>
                            <div class="metric-label mb-1">"VO2 Max"</div>
                            <span class="metric-value text-xl" style=format!("color: {}", theme::CHART_GREEN)>
                                {vo2.map(|v| format!("{:.1}", v)).unwrap_or_else(|| "\u{2014}".into())}
                            </span>
                            <span class="text-dim text-sm">" ml/kg/min"</span>
                        </div>
                        <div class="card" style=format!("border-left: 3px solid {}", theme::CHART_BLUE)>
                            <div class="metric-label mb-1">"Fitness Age"</div>
                            <span class="metric-value text-xl" style=format!("color: {}", theme::CHART_BLUE)>
                                {fitness_age.map(|v| format!("{}", v)).unwrap_or_else(|| "\u{2014}".into())}
                            </span>
                            <span class="text-dim text-sm">" years"</span>
                        </div>
                        <div class="card" style=format!("border-left: 3px solid {}", theme::CHART_ORANGE)>
                            <div class="metric-label mb-1">"Acute Load"</div>
                            <span class="metric-value text-xl" style=format!("color: {}", theme::CHART_ORANGE)>
                                {acute.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "\u{2014}".into())}
                            </span>
                        </div>
                        <div class="card" style=format!("border-left: 3px solid {}", theme::CHART_PURPLE)>
                            <div class="metric-label mb-1">"Training Load"</div>
                            <span class="metric-value text-xl" style=format!("color: {}", theme::CHART_PURPLE)>
                                {load.map(|v| format!("{:.0}", v)).unwrap_or_else(|| "\u{2014}".into())}
                            </span>
                        </div>
                    </div>
                }.into_any()
            }}

            // Training Readiness Components
            {move || {
                let d = state.daily_data.get();
                let feedback_json = d.last().and_then(|day| day.training_readiness_feedback.as_ref())
                    .and_then(|j| serde_json::from_str::<serde_json::Value>(j).ok());
                let feedback_json = match feedback_json {
                    Some(v) => v,
                    None => return view! { <div></div> }.into_any(),
                };
                let components: Vec<(&str, &str, &str)> = vec![
                    ("sleepComponent", "Sleep", theme::CHART_PURPLE),
                    ("recoveryComponent", "Recovery", theme::CHART_GREEN),
                    ("hrvComponent", "HRV", theme::CHART_ORANGE),
                    ("acuteLoadComponent", "Acute Load", theme::CHART_RED),
                    ("chronicLoadComponent", "Chronic Load", theme::CHART_BLUE),
                    ("sleepHistoryComponent", "Sleep History", theme::SLEEP_LIGHT),
                ];
                let cards: Vec<_> = components.iter().filter_map(|(key, label, color)| {
                    let val = &feedback_json[*key];
                    if val.is_null() { return None; }
                    let score = val.as_f64().or_else(|| val["score"].as_f64()).or_else(|| val["value"].as_f64())?;
                    let feedback = val["feedback"].as_str().or_else(|| val["description"].as_str());
                    let score_color = if score >= 66.0 { theme::GOOD } else if score >= 33.0 { theme::CHART_YELLOW } else { theme::WARN };
                    Some(view! {
                        <div class="card" style=format!("border-left: 3px solid {}", color)>
                            <div class="metric-label mb-1">{label.to_string()}</div>
                            <span class="metric-value text-xl" style=format!("color: {}", score_color)>{format!("{:.0}", score)}</span>
                            {feedback.map(|f| view! { <div class="text-dim text-xs mt-1">{f.to_string()}</div> })}
                        </div>
                    })
                }).collect();
                let message = feedback_json["readinessMessage"].as_str()
                    .or_else(|| feedback_json["feedback"].as_str());
                if cards.is_empty() && message.is_none() {
                    return view! { <div></div> }.into_any();
                }
                view! {
                    <div class="mb-6">
                        <h2 class="text-sm font-display font-semibold text-text mb-3">"Readiness Components"</h2>
                        {message.map(|m| view! {
                            <div class="card mb-3" style=format!("border-left: 3px solid {}", theme::INFO)>
                                <span class="text-sm" style=format!("color: {}", theme::TEXT)>{m.to_string()}</span>
                            </div>
                        })}
                        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
                            {cards}
                        </div>
                    </div>
                }.into_any()
            }}

            // Race Predictions
            {move || {
                let ext = state.extended_data.get();
                let e = ext.last();
                let r5k = e.and_then(|e| e.race_5k_secs);
                let r10k = e.and_then(|e| e.race_10k_secs);
                let rhalf = e.and_then(|e| e.race_half_secs);
                let rmarathon = e.and_then(|e| e.race_marathon_secs);
                if r5k.is_none() && r10k.is_none() { return view! { <div></div> }.into_any(); }
                view! {
                    <div class="mb-6">
                        <h2 class="text-sm font-display font-semibold text-text mb-3">"Race Predictions"</h2>
                        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                            {r5k.map(|s| view! {
                                <div class="card" style=format!("border-left: 3px solid {}", theme::CHART_GREEN)>
                                    <div class="metric-label mb-1">"5K"</div>
                                    <span class="metric-value text-xl" style=format!("color: {}", theme::CHART_GREEN)>{fmt_race_time(s)}</span>
                                </div>
                            })}
                            {r10k.map(|s| view! {
                                <div class="card" style=format!("border-left: 3px solid {}", theme::CHART_BLUE)>
                                    <div class="metric-label mb-1">"10K"</div>
                                    <span class="metric-value text-xl" style=format!("color: {}", theme::CHART_BLUE)>{fmt_race_time(s)}</span>
                                </div>
                            })}
                            {rhalf.map(|s| view! {
                                <div class="card" style=format!("border-left: 3px solid {}", theme::CHART_ORANGE)>
                                    <div class="metric-label mb-1">"Half Marathon"</div>
                                    <span class="metric-value text-xl" style=format!("color: {}", theme::CHART_ORANGE)>{fmt_race_time(s)}</span>
                                </div>
                            })}
                            {rmarathon.map(|s| view! {
                                <div class="card" style=format!("border-left: 3px solid {}", theme::CHART_RED)>
                                    <div class="metric-label mb-1">"Marathon"</div>
                                    <span class="metric-value text-xl" style=format!("color: {}", theme::CHART_RED)>{fmt_race_time(s)}</span>
                                </div>
                            })}
                        </div>
                    </div>
                }.into_any()
            }}

            // Trend Charts: VO2 Max + Training Load
            <h2 class="text-sm font-display font-semibold text-text mt-6 mb-3">"Trends"</h2>
            {move || {
                let d = state.daily_data.get();
                if d.is_empty() { return view! { <div></div> }.into_any(); }
                let vo2_series = Series {
                    label: "VO2 Max".into(),
                    points: d.iter().enumerate().filter_map(|(i, d)| d.vo2_max.map(|v| (i as f64, v))).collect(),
                    color: theme::CHART_GREEN.into(), fill: true,
                };
                let load_series = Series {
                    label: "Training Load".into(),
                    points: d.iter().enumerate().filter_map(|(i, d)| d.training_load.map(|v| (i as f64, v))).collect(),
                    color: theme::CHART_ORANGE.into(), fill: true,
                };
                let readiness_series = Series {
                    label: "Readiness".into(),
                    points: d.iter().enumerate().filter_map(|(i, d)| d.training_readiness.map(|v| (i as f64, v))).collect(),
                    color: theme::CHART_BLUE.into(), fill: false,
                };
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                        <TimeseriesChart title="VO2 Max".into()
                            series=vec![vo2_series] unit="ml/kg/min".into() />
                        <TimeseriesChart title="Training Load & Readiness".into()
                            series=vec![load_series, readiness_series] />
                    </div>
                }.into_any()
            }}

            // Fitness Age trend
            {move || {
                let ext = state.extended_data.get();
                let pts: Vec<(f64, f64)> = ext.iter().enumerate()
                    .filter_map(|(i, e)| e.fitness_age.map(|v| (i as f64, v as f64))).collect();
                if pts.is_empty() { return view! { <div></div> }.into_any(); }
                view! {
                    <TimeseriesChart title="Fitness Age".into()
                        series=vec![Series { label: "Fitness Age".into(), points: pts, color: theme::CHART_GREEN.into(), fill: true }]
                        unit="years".into() />
                }.into_any()
            }}
        </div>
    }
}
