use leptos::prelude::*;

/// Big number stat display
#[component]
pub fn StatPanel(
    title: String,
    value: String,
    #[prop(optional)] subtitle: String,
    #[prop(optional)] color: String,
) -> impl IntoView {
    let c = if color.is_empty() { crate::theme::ACCENT.to_string() } else { color };
    view! {
        <div class="bg-surface border border-border rounded-lg p-3 text-center">
            <div class="text-dim text-[0.6rem] uppercase tracking-wider mb-1">{title}</div>
            <div class="text-2xl font-bold" style=format!("color: {}", c)>{value}</div>
            {if !subtitle.is_empty() {
                view! { <div class="text-dim text-xs mt-1">{subtitle}</div> }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </div>
    }
}
