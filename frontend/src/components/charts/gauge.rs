use leptos::prelude::*;

/// Circular gauge panel
#[component]
pub fn Gauge(
    title: String,
    value: Option<f64>,
    #[prop(optional)] unit: String,
    #[prop(optional)] min: f64,
    max: f64,
    #[prop(optional)] thresholds: Vec<(f64, String)>, // (value, color) sorted ascending
) -> impl IntoView {
    let val = value.unwrap_or(0.0);
    let pct = ((val - min) / (max - min)).clamp(0.0, 1.0);

    let color = if thresholds.is_empty() {
        crate::theme::ACCENT.to_string()
    } else {
        let mut c = thresholds.first().map(|(_, c)| c.clone()).unwrap_or(crate::theme::ACCENT.to_string());
        for (t, tc) in &thresholds {
            if val >= *t { c = tc.clone(); }
        }
        c
    };

    let display = if value.is_some() { crate::theme::fmt_val(val) } else { "\u{2014}".to_string() };

    // SVG arc: 180-degree gauge
    let r = 45.0_f64;
    let cx = 60.0_f64;
    let cy = 55.0_f64;
    let start_angle = std::f64::consts::PI;
    let sweep = pct * std::f64::consts::PI;
    let end_angle = start_angle + sweep;

    let x1 = cx + r * start_angle.cos();
    let y1 = cy + r * start_angle.sin();
    let x2 = cx + r * end_angle.cos();
    let y2 = cy + r * end_angle.sin();
    let large_arc = if sweep > std::f64::consts::PI { 1 } else { 0 };

    let bg_path = format!("M {:.1} {:.1} A {:.1} {:.1} 0 1 1 {:.1} {:.1}",
        cx - r, cy, r, r, cx + r, cy);
    let val_path = if pct > 0.001 {
        format!("M {:.1} {:.1} A {:.1} {:.1} 0 {} 1 {:.1} {:.1}",
            x1, y1, r, r, large_arc, x2, y2)
    } else {
        String::new()
    };

    view! {
        <div class="bg-surface border border-border rounded-lg p-3 text-center">
            <div class="text-dim text-[0.6rem] sm:text-[0.7rem] uppercase tracking-wider mb-1">{title}</div>
            <svg viewBox="0 0 120 65" class="w-full min-w-[100px] min-h-[55px] max-w-[140px] mx-auto">
                <path d=bg_path fill="none" stroke=crate::theme::BORDER stroke-width="8" stroke-linecap="round" />
                {if !val_path.is_empty() {
                    view! { <path d=val_path fill="none" stroke=color.clone() stroke-width="8" stroke-linecap="round" /> }.into_any()
                } else {
                    view! { <g></g> }.into_any()
                }}
                <text x="60" y="52" fill=color font-size="18" font-weight="bold" text-anchor="middle">{display}</text>
                <text x="60" y="63" fill=crate::theme::DIM font-size="8" text-anchor="middle">{unit}</text>
            </svg>
        </div>
    }
}
