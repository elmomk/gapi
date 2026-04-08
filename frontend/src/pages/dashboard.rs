use leptos::prelude::*;
use crate::state::AppState;
use crate::theme;
use crate::models::{VitalsData, DailyData};
use crate::components::charts::state_timeline::*;

fn compute_recovery_score(vitals: &VitalsData, _daily: &[DailyData]) -> Option<f64> {
    let hrv_score = match (vitals.hrv_last_night, vitals.baseline_hrv) {
        (Some(today), Some(base)) if base > 0.0 => ((today / base) * 100.0).min(120.0),
        (Some(today), _) => (today / 80.0 * 100.0).min(120.0),
        _ => return None,
    };
    let sleep_score = vitals.sleep_score.map(|s| s as f64).unwrap_or(70.0);
    let rhr_score = match (vitals.resting_heart_rate, vitals.baseline_rhr) {
        (Some(today), Some(base)) if base > 0.0 => ((base / today as f64) * 100.0).min(120.0),
        (Some(today), _) => ((55.0 / today as f64) * 100.0).min(120.0),
        _ => 70.0,
    };
    let stress_score = match vitals.avg_stress {
        Some(s) => ((100.0 - s as f64) / 100.0 * 100.0).max(0.0),
        _ => 70.0,
    };
    Some((hrv_score * 0.4 + sleep_score * 0.3 + rhr_score * 0.2 + stress_score * 0.1).min(100.0))
}

fn detect_anomalies(vitals: &VitalsData) -> Vec<(String, String)> {
    let mut alerts = Vec::new();
    if let (Some(today), Some(base)) = (vitals.hrv_last_night, vitals.baseline_hrv) {
        if today < base * 0.8 {
            alerts.push((format!("HRV dropped {:.0}% below baseline", (1.0 - today / base) * 100.0), "warn".into()));
        }
    }
    if let (Some(today), Some(base)) = (vitals.resting_heart_rate.map(|x| x as f64), vitals.baseline_rhr) {
        if today > base + 5.0 {
            alerts.push((format!("RHR elevated {:.0}bpm above baseline", today - base), "warn".into()));
        }
    }
    if let (Some(today), Some(base)) = (vitals.sleep_score.map(|x| x as f64), vitals.baseline_sleep) {
        if today < base - 15.0 {
            alerts.push((format!("Sleep score dropped {:.0} points", base - today), "warn".into()));
        }
    }
    if let Some(bb) = vitals.body_battery_high {
        if bb < 30 {
            alerts.push(("Body battery critically low - prioritize rest".into(), "warn".into()));
        }
    }
    if let Some(s) = vitals.avg_stress {
        if s > 60 {
            alerts.push((format!("High average stress: {}", s), "info".into()));
        }
    }
    alerts
}

fn compute_weekly_summary(daily: &[DailyData]) -> Option<String> {
    if daily.len() < 7 { return None; }
    let this_week: Vec<&DailyData> = daily.iter().rev().take(7).collect();
    let prev_week: Vec<&DailyData> = daily.iter().rev().skip(7).take(7).collect();
    let sleep_vals: Vec<f64> = this_week.iter().filter_map(|d| d.sleep_duration_secs.map(|s| s as f64 / 3600.0)).collect();
    let avg_sleep = if sleep_vals.is_empty() { None } else { Some(sleep_vals.iter().sum::<f64>() / sleep_vals.len() as f64) };
    let prev_sleep_vals: Vec<f64> = prev_week.iter().filter_map(|d| d.sleep_duration_secs.map(|s| s as f64 / 3600.0)).collect();
    let prev_avg_sleep = if prev_sleep_vals.is_empty() { None } else { Some(prev_sleep_vals.iter().sum::<f64>() / prev_sleep_vals.len() as f64) };
    let sleep_delta = match (avg_sleep, prev_avg_sleep) {
        (Some(a), Some(b)) => { let d = a - b; if d >= 0.0 { format!(" (+{:.1}h)", d) } else { format!(" ({:.1}h)", d) } }
        _ => String::new(),
    };
    let training_days = this_week.iter().filter(|d| d.activities_count.unwrap_or(0) > 0).count();
    let hrv_7: Vec<f64> = this_week.iter().filter_map(|d| d.hrv_last_night).collect();
    let hrv_prev: Vec<f64> = prev_week.iter().filter_map(|d| d.hrv_last_night).collect();
    let hrv_trend = match (
        if hrv_7.is_empty() { None } else { Some(hrv_7.iter().sum::<f64>() / hrv_7.len() as f64) },
        if hrv_prev.is_empty() { None } else { Some(hrv_prev.iter().sum::<f64>() / hrv_prev.len() as f64) },
    ) {
        (Some(a), Some(b)) if a > b * 1.05 => "\u{2191} up",
        (Some(a), Some(b)) if a < b * 0.95 => "\u{2193} down",
        _ => "\u{2194} stable",
    };
    let total_steps: i64 = this_week.iter().filter_map(|d| d.steps).sum();
    let steps_str = if total_steps >= 1000 { format!("{}k steps", total_steps / 1000) } else { format!("{} steps", total_steps) };
    let weight_str = match (this_week.last().and_then(|d| d.weight_grams), this_week.first().and_then(|d| d.weight_grams)) {
        (Some(f), Some(l)) => { let d = (l - f) / 1000.0; if d >= 0.0 { format!(", wt +{:.1}kg", d) } else { format!(", wt {:.1}kg", d) } }
        _ => String::new(),
    };
    let sleep_str = avg_sleep.map(|s| format!("{:.1}h sleep{}", s, sleep_delta)).unwrap_or_else(|| "no sleep data".into());
    Some(format!("{}, HRV {}, {} training days, {}{}", sleep_str, hrv_trend, training_days, steps_str, weight_str))
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    view! {
        <div class="animate-slide-up">
            <h1 class="page-title">"Dashboard"</h1>
            <p class="page-subtitle">{move || {
                let sel = state.selected_date.get();
                let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                let date_label = if sel == today {
                    chrono::Local::now().format("%A, %B %d").to_string()
                } else if let Ok(d) = chrono::NaiveDate::parse_from_str(&sel, "%Y-%m-%d") {
                    d.format("%A, %B %d").to_string()
                } else {
                    sel.clone()
                };
                let sync_label = {
                    let users = state.users.get();
                    let uid = state.user_id.get();
                    users.iter().find(|u| u.user_id == uid)
                        .and_then(|u| u.last_sync_at)
                        .map(|ts| {
                            let elapsed = chrono::Utc::now().timestamp() as f64 - ts;
                            let mins = (elapsed / 60.0) as i64;
                            if mins < 1 { "just now".to_string() }
                            else if mins < 60 { format!("{}m ago", mins) }
                            else if mins < 1440 { format!("{}h ago", mins / 60) }
                            else { format!("{}d ago", mins / 1440) }
                        })
                        .unwrap_or_default()
                };
                if sync_label.is_empty() {
                    format!("// {}", date_label)
                } else {
                    format!("// {} · synced {}", date_label, sync_label)
                }
            }}</p>

            // Recovery Score Card
            {move || {
                let vitals = state.vitals.get();
                let daily = state.daily_data.get();
                vitals.as_ref().and_then(|v| {
                    compute_recovery_score(v, &daily).map(|score| {
                        let (color, label) = if score > 70.0 {
                            (theme::GOOD, "Recovered")
                        } else if score >= 50.0 {
                            (theme::CHART_YELLOW, "Moderate")
                        } else {
                            (theme::WARN, "Low Recovery")
                        };
                        view! {
                            <div class="card mb-6" style=format!("border-left: 3px solid {}", color)>
                                <div class="flex items-center justify-between">
                                    <div>
                                        <div class="metric-label mb-1">"Recovery Score"</div>
                                        <div class="flex items-baseline gap-2">
                                            <span class="text-4xl font-display font-bold" style=format!("color: {}", color)>
                                                {format!("{:.0}", score)}
                                            </span>
                                            <span class="text-dim text-sm font-display">"/100"</span>
                                        </div>
                                        <div class="text-sm mt-1" style=format!("color: {}", color)>{label}</div>
                                    </div>
                                    <div class="text-dim text-xs text-right" style="max-width: 200px">
                                        "HRV 40% + Sleep 30% + RHR 20% + Stress 10%"
                                    </div>
                                </div>
                            </div>
                        }
                    })
                })
            }}

            // Loading skeletons
            <Show when=move || state.loading.get() && state.vitals.get().is_none()>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
                    <div class="card"><div class="skeleton h-4 w-16 mb-3"></div><div class="skeleton h-8 w-24 mb-2"></div><div class="skeleton h-3 w-20"></div></div>
                    <div class="card"><div class="skeleton h-4 w-16 mb-3"></div><div class="skeleton h-8 w-24 mb-2"></div><div class="skeleton h-3 w-20"></div></div>
                    <div class="card"><div class="skeleton h-4 w-16 mb-3"></div><div class="skeleton h-8 w-24 mb-2"></div><div class="skeleton h-3 w-20"></div></div>
                    <div class="card"><div class="skeleton h-4 w-16 mb-3"></div><div class="skeleton h-8 w-24 mb-2"></div><div class="skeleton h-3 w-20"></div></div>
                </div>
            </Show>

            // Vitals grid (2 rows of 4)
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
                        <VitalCard label="Body Battery" value={
                            let latest_bb = state.intraday_stress.get().iter().rev()
                                .find_map(|p| p.body_battery)
                                .map(|x| x as f64);
                            latest_bb.or(v.body_battery_high.map(|x| x as f64))
                        } unit="%" baseline=v.baseline_battery higher_is_better=true color=theme::BB_CHARGED />
                        <VitalCard label="Readiness" value=v.training_readiness unit="" baseline=None higher_is_better=true color=theme::CHART_BLUE />
                        <VitalCard label="Steps" value=v.steps.map(|x| x as f64) unit="" baseline=None higher_is_better=true color=theme::CHART_GREEN />
                    </div>
                }
            })}

            // Compact status row: HRV status + Sleep Debt + Training Phase
            {move || {
                let d = state.daily_data.get();
                let ext = state.extended_data.get();
                let hrv_status = d.last().and_then(|d| d.hrv_status.clone());
                let debt = state.vitals.get().and_then(|v| v.sleep_debt_hours);
                let phase = ext.last().and_then(|e| e.training_status_phase.clone());

                let has_any = hrv_status.is_some() || debt.is_some() || phase.is_some();
                if !has_any { return view! { <div></div> }.into_any(); }

                view! {
                    <div class="flex flex-wrap gap-3 mb-6">
                        {hrv_status.map(|status| {
                            let color = match status.as_str() {
                                "BALANCED" | "HIGH" => theme::GOOD,
                                "UNBALANCED" | "LOW" => theme::WARN,
                                _ => theme::CHART_YELLOW,
                            };
                            view! {
                                <div class="card flex-1 min-w-[140px]" style=format!("border-left: 3px solid {}", color)>
                                    <span class="metric-label">"HRV "</span>
                                    <span class="text-sm font-display font-semibold" style=format!("color: {}", color)>{status}</span>
                                </div>
                            }
                        })}
                        {debt.map(|d| {
                            let color = if d <= 2.0 { theme::GOOD } else if d <= 5.0 { theme::CHART_YELLOW } else { theme::WARN };
                            let label = if d <= 0.0 { "surplus" } else { "deficit" };
                            view! {
                                <div class="card flex-1 min-w-[140px]" style=format!("border-left: 3px solid {}", color)>
                                    <span class="metric-label">"Sleep Debt "</span>
                                    <span class="text-sm font-display font-bold" style=format!("color: {}", color)>{format!("{:.1}h", d)}</span>
                                    <span class="text-dim text-xs">" "{label}</span>
                                </div>
                            }
                        })}
                        {phase.map(|p| {
                            let color = match p.as_str() {
                                "PRODUCTIVE" | "PEAKING" => theme::GOOD,
                                "RECOVERY" | "MAINTAINING" => theme::CHART_YELLOW,
                                "UNPRODUCTIVE" | "DETRAINING" | "OVERREACHING" => theme::WARN,
                                _ => theme::INFO,
                            };
                            view! {
                                <div class="card flex-1 min-w-[140px]" style=format!("border-left: 3px solid {}", color)>
                                    <span class="metric-label">"Training "</span>
                                    <span class="text-sm font-display font-semibold" style=format!("color: {}", color)>{p}</span>
                                </div>
                            }
                        })}
                    </div>
                }.into_any()
            }}

            // Weekly Summary
            {move || {
                let daily = state.daily_data.get();
                compute_weekly_summary(&daily).map(|summary| {
                    view! {
                        <div class="card mb-6" style=format!("border-left: 3px solid {}", theme::INFO)>
                            <div class="metric-label mb-1">"This Week"</div>
                            <div class="text-sm" style=format!("color: {}", theme::TEXT)>{summary}</div>
                        </div>
                    }
                })
            }}

            // Anomaly Alerts
            {move || {
                state.vitals.get().map(|v| {
                    let alerts = detect_anomalies(&v);
                    if alerts.is_empty() { return view! { <div></div> }.into_any(); }
                    view! {
                        <div class="mb-6">
                            <h2 class="text-sm font-display font-semibold text-text mb-3">"Alerts"</h2>
                            <div class="grid gap-2">
                                {alerts.into_iter().map(|(msg, severity)| {
                                    let color = if severity == "warn" { theme::WARN } else { theme::INFO };
                                    view! {
                                        <div class="card" style=format!("border-left: 3px solid {}", color)>
                                            <span class="text-sm" style=format!("color: {}", color)>{msg}</span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_any()
                })
            }}

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
