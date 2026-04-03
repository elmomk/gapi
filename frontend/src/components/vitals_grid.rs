use leptos::prelude::*;
use crate::models::VitalsData;

#[component]
pub fn VitalsGrid(vitals: ReadSignal<Option<VitalsData>>) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-3 mb-6">
            {move || vitals.get().map(|v| {
                vec![
                    vital_card("HRV", v.hrv_last_night, "ms", v.baseline_hrv, true),
                    vital_card("Resting HR", v.resting_heart_rate.map(|x| x as f64), "bpm", v.baseline_rhr, false),
                    vital_card("Sleep Score", v.sleep_score.map(|x| x as f64), "", v.baseline_sleep, true),
                    vital_card("Sleep", v.sleep_hours, "hrs", None, true),
                    vital_card("Stress", v.avg_stress.map(|x| x as f64), "", v.baseline_stress, false),
                    vital_card("Body Battery", v.body_battery_high.map(|x| x as f64), "", v.baseline_battery, true),
                    vital_card("Readiness", v.training_readiness, "", None, true),
                    vital_card("Steps", v.steps.map(|x| x as f64), "", None, true),
                ].into_iter().collect::<Vec<_>>()
            })}
        </div>
    }
}

fn vital_card(
    label: &str,
    value: Option<f64>,
    unit: &str,
    baseline: Option<f64>,
    higher_is_better: bool,
) -> impl IntoView {
    let display_val = match value {
        Some(v) if v == v.floor() => format!("{:.0}", v),
        Some(v) => format!("{:.1}", v),
        None => "\u{2014}".to_string(),
    };

    let (delta_class, delta_text) = match (value, baseline) {
        (Some(today), Some(base)) if base > 0.0 => {
            let pct = ((today - base) / base * 100.0) as i64;
            let better = if higher_is_better { today >= base * 0.9 } else { today <= base * 1.1 };
            let cls = if better { "text-good" } else { "text-warn" };
            let sign = if pct > 0 { "+" } else { "" };
            (cls, format!("{sign}{pct}% vs baseline"))
        }
        _ => ("text-info", "no baseline".to_string()),
    };

    let baseline_text = match baseline {
        Some(b) if b == b.floor() => format!("baseline: {:.0}", b),
        Some(b) => format!("baseline: {:.1}", b),
        None => String::new(),
    };

    let label = label.to_string();
    let unit = unit.to_string();
    let delta_class = delta_class.to_string();

    view! {
        <div class="bg-surface border border-border rounded-lg p-4">
            <div class="text-dim text-[0.65rem] uppercase tracking-wider mb-1">{label}</div>
            <div class="text-3xl font-bold">
                {display_val}
                <span class="text-dim text-xs ml-1">{unit}</span>
            </div>
            <div class="text-dim text-xs mt-1">{baseline_text}</div>
            <div class=format!("{delta_class} text-xs mt-0.5")>{delta_text}</div>
        </div>
    }
}
