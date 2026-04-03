use leptos::prelude::*;

/// A series to plot
#[derive(Clone)]
pub struct Series {
    pub label: String,
    pub points: Vec<(f64, f64)>, // (x, y) where x is typically a timestamp or index
    pub color: String,
    pub fill: bool,
}

/// Line/area timeseries chart
#[component]
pub fn TimeseriesChart(
    title: String,
    series: Vec<Series>,
    #[prop(optional)] x_labels: Vec<String>,
    #[prop(optional)] unit: String,
    #[prop(optional)] height: Option<f64>,
) -> impl IntoView {
    let h = height.unwrap_or(180.0);
    let w = 800.0_f64;
    let margin = (30.0_f64, 10.0_f64, 20.0_f64, 40.0_f64); // top, right, bottom, left

    if series.is_empty() || series.iter().all(|s| s.points.is_empty()) {
        return view! { <div></div> }.into_any();
    }

    let plot_w = w - margin.3 - margin.1;
    let plot_h = h - margin.0 - margin.2;

    // Find global bounds
    let all_y: Vec<f64> = series.iter().flat_map(|s| s.points.iter().map(|(_, y)| *y)).collect();
    let data_y_min = all_y.iter().cloned().fold(f64::INFINITY, f64::min);
    let data_y_max = all_y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_range = (data_y_max - data_y_min).max(1.0);
    // If the range is small relative to the values (like weight: 93-95 out of 95),
    // zoom into the data range with 10% padding. Otherwise start from 0.
    let (y_min, y_max) = if data_y_min > 0.0 && y_range / data_y_max < 0.3 {
        // Tight range: zoom in with padding
        let pad = y_range * 0.15;
        (data_y_min - pad, data_y_max + pad)
    } else {
        // Wide range: start from 0
        (0.0_f64.min(data_y_min), data_y_max + y_range * 0.05)
    };

    let all_x: Vec<f64> = series.iter().flat_map(|s| s.points.iter().map(|(x, _)| *x)).collect();
    let x_min = all_x.iter().cloned().fold(f64::INFINITY, f64::min);
    let x_max = all_x.iter().cloned().fold(f64::NEG_INFINITY, f64::max).max(x_min + 1.0);

    let sx = move |x: f64| margin.3 + (x - x_min) / (x_max - x_min) * plot_w;
    let sy = move |y: f64| margin.0 + plot_h - (y - y_min) / (y_max - y_min) * plot_h;

    // Build SVG paths
    let paths: Vec<_> = series.iter().map(|s| {
        if s.points.is_empty() { return (String::new(), String::new(), s.color.clone(), s.fill, s.label.clone()); }
        let mut line = String::new();
        for (i, (x, y)) in s.points.iter().enumerate() {
            let cmd = if i == 0 { "M" } else { "L" };
            line.push_str(&format!("{}{:.1},{:.1} ", cmd, sx(*x), sy(*y)));
        }
        let fill_path = if s.fill {
            let first_x = sx(s.points.first().unwrap().0);
            let last_x = sx(s.points.last().unwrap().0);
            let bottom = sy(y_min);
            format!("{}L{:.1},{:.1} L{:.1},{:.1} Z", line, last_x, bottom, first_x, bottom)
        } else {
            String::new()
        };
        (line, fill_path, s.color.clone(), s.fill, s.label.clone())
    }).collect();

    // Y-axis labels (5 ticks)
    let y_ticks: Vec<(f64, String)> = (0..=4).map(|i| {
        let v = y_min + (y_max - y_min) * i as f64 / 4.0;
        (sy(v), crate::theme::fmt_val(v))
    }).collect();

    // Compute averages for legend
    let legends: Vec<(String, String, String)> = series.iter().map(|s| {
        let avg = if s.points.is_empty() { 0.0 } else { s.points.iter().map(|(_, y)| y).sum::<f64>() / s.points.len() as f64 };
        (s.label.clone(), s.color.clone(), format!("avg: {}", crate::theme::fmt_val(avg)))
    }).collect();

    view! {
        <div class="bg-surface border border-border rounded-lg p-4 mb-3">
            <div class="flex justify-between items-center mb-2">
                <span class="text-text text-sm">{title}</span>
                <div class="flex gap-4">
                    {legends.iter().map(|(label, color, avg)| {
                        let c = color.clone();
                        let l = label.clone();
                        let a = avg.clone();
                        view! {
                            <span class="text-[0.65rem] flex items-center gap-1">
                                <span class="w-2 h-2 rounded-full inline-block" style=format!("background: {}", c)></span>
                                <span class="text-dim">{l}</span>
                                <span class="text-text">{a}</span>
                            </span>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
            <svg viewBox=format!("0 0 {} {}", w, h) class="w-full" style=format!("height: {}px", h)>
                // Grid lines
                {y_ticks.iter().map(|(y, _)| {
                    view! { <line x1=margin.3 y1=*y x2=(w - margin.1) y2=*y stroke=crate::theme::BORDER stroke-width="0.5" /> }
                }).collect::<Vec<_>>()}
                // Y labels
                {y_ticks.iter().map(|(y, label)| {
                    let l = label.clone();
                    view! { <text x=(margin.3 - 4.0) y=*y fill=crate::theme::DIM font-size="9" text-anchor="end" dominant-baseline="middle">{l}</text> }
                }).collect::<Vec<_>>()}
                // Fill areas
                {paths.iter().filter(|(_, fp, _, fill, _)| *fill && !fp.is_empty()).map(|(_, fp, color, _, _)| {
                    view! { <path d=fp.clone() fill=color.clone() opacity="0.15" /> }
                }).collect::<Vec<_>>()}
                // Lines
                {paths.iter().filter(|(lp, _, _, _, _)| !lp.is_empty()).map(|(lp, _, color, _, _)| {
                    view! { <path d=lp.clone() fill="none" stroke=color.clone() stroke-width="1.5" /> }
                }).collect::<Vec<_>>()}
            </svg>
        </div>
    }.into_any()
}
