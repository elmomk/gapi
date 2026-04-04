use leptos::prelude::*;
use crate::state::AppState;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let state = expect_context::<AppState>();

    let save_and_reload = move || {
        crate::save_setting("garmin_api_url", &state.api_url.get_untracked());
        crate::save_setting("garmin_api_key", &state.api_key.get_untracked());
        crate::save_setting("garmin_user_id", &state.user_id.get_untracked());
        let s = state.clone();
        leptos::task::spawn_local(async move { s.load_all().await; });
    };

    let sync_now = move || {
        let s = state.clone();
        leptos::task::spawn_local(async move { s.trigger_sync().await; });
    };

    view! {
        <div class="animate-slide-up max-w-2xl">
            <h1 class="page-title">"Settings"</h1>
            <p class="page-subtitle">"API configuration and sync controls"</p>

            <div class="card mb-4">
                <h2 class="font-display font-semibold text-sm mb-4">"API Connection"</h2>
                <div class="space-y-4">
                    <div>
                        <label class="metric-label block mb-1.5">"API URL"</label>
                        <input class="input-field font-mono"
                            prop:value=move || state.api_url.get()
                            on:input=move |e| {
                                use wasm_bindgen::JsCast;
                                state.api_url.set(e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value());
                            }
                        />
                    </div>
                    <div>
                        <label class="metric-label block mb-1.5">"API Key"</label>
                        <input type="password" class="input-field font-mono"
                            prop:value=move || state.api_key.get()
                            on:input=move |e| {
                                use wasm_bindgen::JsCast;
                                state.api_key.set(e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value());
                            }
                        />
                    </div>
                    <div>
                        <label class="metric-label block mb-1.5">"User ID"</label>
                        <input class="input-field font-mono"
                            prop:value=move || state.user_id.get()
                            on:input=move |e| {
                                use wasm_bindgen::JsCast;
                                state.user_id.set(e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value());
                            }
                        />
                    </div>
                    <div class="flex gap-3">
                        <button class="btn-primary" on:click=move |_| save_and_reload()>"Save & Reload"</button>
                        <button class="btn-secondary" on:click=move |_| sync_now()>"Sync Now"</button>
                    </div>
                </div>
            </div>

            // Health targets
            <div class="card mb-4">
                <h2 class="font-display font-semibold text-sm mb-4">"Health Targets"</h2>
                <div>
                    <label class="metric-label block mb-1.5">"Sleep Target"</label>
                    <div class="flex items-center gap-3">
                        <input type="range" min="5" max="10" step="0.5"
                            class="flex-1 accent-accent"
                            prop:value=move || format!("{}", state.sleep_target_hours.get())
                            on:input=move |e| {
                                use wasm_bindgen::JsCast;
                                if let Ok(v) = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value().parse::<f64>() {
                                    state.sleep_target_hours.set(v);
                                    crate::save_setting("sleep_target_hours", &format!("{}", v));
                                }
                            }
                        />
                        <span class="text-accent font-bold text-lg min-w-[50px] text-center">
                            {move || format!("{:.0}h", state.sleep_target_hours.get())}
                        </span>
                    </div>
                    <div class="text-dim text-xs mt-1">"Used for sleep debt calculation"</div>
                </div>
            </div>

            // Status
            {move || {
                let (msg, t) = state.status.get();
                if msg.is_empty() { return view! { <div></div> }.into_any(); }
                let cls = match t.as_str() {
                    "ok" => "border-good/30 text-good bg-good/10",
                    "err" => "border-warn/30 text-warn bg-warn/10",
                    _ => "border-info/30 text-info bg-info/10",
                };
                view! {
                    <div class=format!("card-flat border {} text-sm", cls)>{msg}</div>
                }.into_any()
            }}

            // About
            <div class="card mt-4">
                <h2 class="font-display font-semibold text-sm mb-2">"About"</h2>
                <div class="text-dim text-sm space-y-1">
                    <p>"Garmin Dashboard v0.2.0"</p>
                    <p>"Built with Leptos + Tailwind CSS"</p>
                    <p>"Data from garmin_api microservice"</p>
                </div>
            </div>
        </div>
    }
}
