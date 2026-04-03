use leptos::prelude::*;

#[derive(Clone)]
pub struct PieSlice {
    pub label: String,
    pub value: f64,
    pub color: String,
}

/// Donut pie chart
#[component]
pub fn PieChart(
    title: String,
    slices: Vec<PieSlice>,
    #[prop(optional)] format_fn: Option<fn(f64) -> String>,
) -> impl IntoView {
    if slices.is_empty() || slices.iter().all(|s| s.value <= 0.0) {
        return view! { <div></div> }.into_any();
    }

    let total: f64 = slices.iter().map(|s| s.value.max(0.0)).sum();
    let fmt = format_fn.unwrap_or(crate::theme::fmt_val);

    let cx = 60.0_f64;
    let cy = 60.0_f64;
    let r = 45.0_f64;
    let inner_r = 28.0_f64;

    let mut paths = Vec::new();
    let mut angle = -std::f64::consts::FRAC_PI_2; // start at top

    for s in &slices {
        if s.value <= 0.0 { continue; }
        let sweep = s.value / total * 2.0 * std::f64::consts::PI;
        let end_angle = angle + sweep;
        let large = if sweep > std::f64::consts::PI { 1 } else { 0 };

        let ox1 = cx + r * angle.cos();
        let oy1 = cy + r * angle.sin();
        let ox2 = cx + r * end_angle.cos();
        let oy2 = cy + r * end_angle.sin();
        let ix1 = cx + inner_r * end_angle.cos();
        let iy1 = cy + inner_r * end_angle.sin();
        let ix2 = cx + inner_r * angle.cos();
        let iy2 = cy + inner_r * angle.sin();

        let d = format!(
            "M {:.1} {:.1} A {:.1} {:.1} 0 {} 1 {:.1} {:.1} L {:.1} {:.1} A {:.1} {:.1} 0 {} 0 {:.1} {:.1} Z",
            ox1, oy1, r, r, large, ox2, oy2,
            ix1, iy1, inner_r, inner_r, large, ix2, iy2,
        );
        paths.push((d, s.color.clone()));
        angle = end_angle;
    }

    view! {
        <div class="bg-surface border border-border rounded-lg p-3">
            <div class="text-dim text-[0.6rem] uppercase tracking-wider mb-2 text-center">{title}</div>
            <div class="flex items-center gap-3">
                <svg viewBox="0 0 120 120" class="w-24 h-24 flex-shrink-0">
                    {paths.iter().map(|(d, c)| {
                        view! { <path d=d.clone() fill=c.clone() /> }
                    }).collect::<Vec<_>>()}
                </svg>
                <div class="flex flex-col gap-1">
                    {slices.iter().filter(|s| s.value > 0.0).map(|s| {
                        let c = s.color.clone();
                        let l = s.label.clone();
                        let v = fmt(s.value);
                        view! {
                            <div class="flex items-center gap-1 text-[0.65rem]">
                                <span class="w-2 h-2 rounded-sm flex-shrink-0" style=format!("background: {}", c)></span>
                                <span class="text-dim">{l}</span>
                                <span class="text-text">{v}</span>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }.into_any()
}
