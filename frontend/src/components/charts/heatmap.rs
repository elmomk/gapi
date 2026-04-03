use leptos::prelude::*;

/// Grid heatmap (e.g., hour x day)
#[component]
pub fn Heatmap(
    title: String,
    rows: Vec<String>,       // row labels (e.g., hours)
    cols: Vec<String>,       // col labels (e.g., dates)
    values: Vec<Vec<f64>>,   // values[row][col]
    #[prop(optional)] low_color: String,
    #[prop(optional)] high_color: String,
) -> impl IntoView {
    if rows.is_empty() || cols.is_empty() || values.is_empty() {
        return view! { <div></div> }.into_any();
    }

    let low = if low_color.is_empty() { crate::theme::BG.to_string() } else { low_color };
    let high = if high_color.is_empty() { crate::theme::ACCENT.to_string() } else { high_color };

    let all_vals: Vec<f64> = values.iter().flat_map(|r| r.iter().copied()).filter(|v| *v > 0.0).collect();
    let max_val = all_vals.iter().cloned().fold(f64::MIN, f64::max).max(0.001);

    let show_col_every = if cols.len() > 30 { (cols.len() / 15).max(1) } else { 1 };

    view! {
        <div class="bg-surface border border-border rounded-lg p-4 mb-3">
            <div class="text-text text-sm mb-2">{title}</div>
            <div class="overflow-x-auto">
                <div class="flex gap-[1px]">
                    // Row labels column
                    <div class="flex flex-col gap-[1px] flex-shrink-0">
                        <div class="h-3"></div>
                        {rows.iter().map(|label| {
                            let l = label.clone();
                            view! { <div class="h-3 text-[0.45rem] text-dim text-right pr-1 leading-none">{l}</div> }
                        }).collect::<Vec<_>>()}
                    </div>
                    // Grid
                    <div class="flex gap-[1px] flex-1">
                        {cols.iter().enumerate().map(|(ci, col_label)| {
                            let label = if ci % show_col_every == 0 { col_label.get(5..).unwrap_or(col_label).to_string() } else { String::new() };
                            view! {
                                <div class="flex flex-col gap-[1px] flex-1 min-w-[3px]">
                                    <div class="h-3 text-[0.4rem] text-dim text-center leading-none overflow-hidden">{label}</div>
                                    {rows.iter().enumerate().map(|(ri, _)| {
                                        let v = values.get(ri).and_then(|r| r.get(ci)).copied().unwrap_or(0.0);
                                        let t = (v / max_val).clamp(0.0, 1.0);
                                        let color = crate::theme::lerp_color(&low, &high, t);
                                        let style = format!("background: {}; height: 12px;", color);
                                        view! { <div class="rounded-[1px]" style=style></div> }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </div>
        </div>
    }.into_any()
}
