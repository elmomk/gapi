mod api;
mod models;
mod theme;
mod components;

use leptos::prelude::*;
use models::*;
use components::config_bar::ConfigBar;
use components::vitals_grid::VitalsGrid;
use components::sections::trends::TrendsSection;
use components::sections::intraday::IntradaySection;
use components::sections::recent_activities::RecentActivitiesSection;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).ok();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (api_url, set_api_url) = signal(load_setting("garmin_api_url", ""));
    let (api_key, set_api_key) = signal(load_setting("garmin_api_key", ""));
    let (user_id, set_user_id) = signal(load_setting("garmin_user_id", ""));
    let (days, set_days) = signal(30i64);
    let (status_msg, set_status_msg) = signal(String::new());
    let (status_type, set_status_type) = signal("loading".to_string());

    // Data signals
    let (vitals, set_vitals) = signal(None::<VitalsData>);
    let (daily_data, set_daily_data) = signal(Vec::<DailyData>::new());
    let (loading, set_loading) = signal(false);

    // Intraday signals
    let (intraday_hr, set_intraday_hr) = signal(Vec::<IntradayPoint>::new());
    let (intraday_stress, set_intraday_stress) = signal(Vec::<StressPoint>::new());
    let (intraday_hrv, set_intraday_hrv) = signal(Vec::<HrvReading>::new());
    let (intraday_sleep, set_intraday_sleep) = signal(Vec::<SleepEpoch>::new());
    let (intraday_resp, set_intraday_resp) = signal(Vec::<IntradayPointF64>::new());

    // Section collapse state
    let (show_intraday, set_show_intraday) = signal(true);
    let (show_activities, set_show_activities) = signal(true);
    let (show_trends, set_show_trends) = signal(true);

    // Save settings
    Effect::new(move || { save_setting("garmin_api_url", &api_url.get()); });
    Effect::new(move || { save_setting("garmin_api_key", &api_key.get()); });
    Effect::new(move || { save_setting("garmin_user_id", &user_id.get()); });

    // Load all data
    let load_all = Action::new_local(move |_: &()| {
        let url = api_url.get(); let key = api_key.get(); let uid = user_id.get(); let d = days.get();
        async move {
            set_loading.set(true);
            set_status_msg.set("Loading...".into());
            set_status_type.set("loading".into());

            // Fetch daily + vitals
            let vitals_res = api::fetch_vitals(&url, &key, &uid).await;
            let daily_res = api::fetch_daily_range(&url, &key, &uid, d).await;

            match vitals_res {
                Ok(v) => set_vitals.set(Some(v)),
                Err(e) => { set_status_msg.set(format!("Error: {e}")); set_status_type.set("err".into()); set_loading.set(false); return; }
            }
            match daily_res {
                Ok(data) => { let n = data.len(); set_daily_data.set(data); set_status_msg.set(format!("Loaded {n} days")); set_status_type.set("ok".into()); }
                Err(e) => { set_status_msg.set(format!("Error: {e}")); set_status_type.set("err".into()); }
            }

            // Fetch intraday for today
            let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
            set_intraday_hr.set(api::fetch_intraday_hr(&url, &key, &uid, &today).await.unwrap_or_default());
            set_intraday_stress.set(api::fetch_intraday_stress(&url, &key, &uid, &today).await.unwrap_or_default());
            set_intraday_hrv.set(api::fetch_intraday_hrv(&url, &key, &uid, &today).await.unwrap_or_default());
            set_intraday_sleep.set(api::fetch_intraday_sleep(&url, &key, &uid, &today).await.unwrap_or_default());
            set_intraday_resp.set(api::fetch_intraday_respiration(&url, &key, &uid, &today).await.unwrap_or_default());

            set_loading.set(false);
        }
    });

    let trigger_sync = Action::new_local(move |_: &()| {
        let url = api_url.get(); let key = api_key.get(); let uid = user_id.get();
        async move {
            set_status_msg.set("Syncing...".into()); set_status_type.set("loading".into());
            match api::trigger_sync(&url, &key, &uid).await {
                Ok(msg) => { set_status_msg.set(msg); set_status_type.set("ok".into()); }
                Err(e) => { set_status_msg.set(format!("Sync failed: {e}")); set_status_type.set("err".into()); }
            }
        }
    });

    let load_charts = Action::new_local(move |_: &()| {
        let url = api_url.get(); let key = api_key.get(); let uid = user_id.get(); let d = days.get();
        async move {
            if let Ok(data) = api::fetch_daily_range(&url, &key, &uid, d).await { set_daily_data.set(data); }
        }
    });

    // Bootstrap: load config.json and auto-load
    Effect::new(move || {
        if api_key.get().is_empty() || user_id.get().is_empty() {
            leptos::task::spawn_local(async move {
                let origin = web_sys::window().and_then(|w| w.location().origin().ok()).unwrap_or_default();
                let url = format!("{}/config.json", origin);
                if let Ok(resp) = reqwest::Client::new().get(&url).send().await {
                    if let Ok(cfg) = resp.json::<serde_json::Value>().await {
                        if let Some(u) = cfg["api_url"].as_str() { if api_url.get().is_empty() || api_url.get().contains("localhost") { set_api_url.set(u.to_string()); } }
                        if let Some(k) = cfg["api_key"].as_str() { if api_key.get().is_empty() { set_api_key.set(k.to_string()); } }
                        if let Some(id) = cfg["user_id"].as_str() { if user_id.get().is_empty() { set_user_id.set(id.to_string()); } }
                        if !api_key.get().is_empty() && !user_id.get().is_empty() { load_all.dispatch(()); }
                    }
                }
            });
        } else {
            load_all.dispatch(());
        }
    });

    view! {
        <div class="p-4 max-w-screen-2xl mx-auto">
            <h1 class="text-accent text-2xl font-bold mb-1">"GARMIN DASHBOARD"</h1>
            <p class="text-dim text-sm mb-4">"Health analytics powered by garmin_api"</p>

            <ConfigBar
                api_url=api_url set_api_url=set_api_url
                api_key=api_key set_api_key=set_api_key
                user_id=user_id set_user_id=set_user_id
                on_load=move || { load_all.dispatch(()); }
                on_sync=move || { trigger_sync.dispatch(()); }
                loading=loading
            />

            // Status bar
            <Show when=move || !status_msg.get().is_empty()>
                <div class=move || format!("px-4 py-2 rounded text-sm mb-4 border {}", match status_type.get().as_str() {
                    "ok" => "bg-good/10 text-good border-good/30",
                    "err" => "bg-warn/10 text-warn border-warn/30",
                    _ => "bg-info/10 text-info border-info/30",
                })>{move || status_msg.get()}</div>
            </Show>

            // Vitals cards
            <Show when=move || vitals.get().is_some()>
                <h2 class="text-dim text-xs uppercase tracking-widest mb-3 mt-6">"Today's Vitals"</h2>
                <VitalsGrid vitals=vitals />
            </Show>

            // Section 1: Intraday
            <Show when=move || !intraday_hr.get().is_empty() || !intraday_stress.get().is_empty()>
                <button class="text-dim text-xs uppercase tracking-widest mt-6 mb-3 flex items-center gap-2 hover:text-text"
                    on:click=move |_| set_show_intraday.set(!show_intraday.get())>
                    <span>{move || if show_intraday.get() { "\u{25BC}" } else { "\u{25B6}" }}</span>
                    "Intraday Health Stats"
                </button>
                <Show when=move || show_intraday.get()>
                    <IntradaySection hr_data=intraday_hr stress_data=intraday_stress hrv_data=intraday_hrv
                        sleep_data=intraday_sleep resp_data=intraday_resp vitals=vitals />
                </Show>
            </Show>

            // Section 2: Recent Activities
            <Show when=move || !daily_data.get().is_empty()>
                <button class="text-dim text-xs uppercase tracking-widest mt-6 mb-3 flex items-center gap-2 hover:text-text"
                    on:click=move |_| set_show_activities.set(!show_activities.get())>
                    <span>{move || if show_activities.get() { "\u{25BC}" } else { "\u{25B6}" }}</span>
                    "Recent Activities"
                </button>
                <Show when=move || show_activities.get()>
                    <RecentActivitiesSection data=daily_data />
                </Show>
            </Show>

            // Section 3: Trends (with range selector)
            <Show when=move || !daily_data.get().is_empty()>
                <div class="flex items-center gap-3 mt-6 mb-3">
                    <button class="text-dim text-xs uppercase tracking-widest flex items-center gap-2 hover:text-text"
                        on:click=move |_| set_show_trends.set(!show_trends.get())>
                        <span>{move || if show_trends.get() { "\u{25BC}" } else { "\u{25B6}" }}</span>
                        "Long-term Trends"
                    </button>
                    <div class="flex gap-1">
                        {[7, 14, 30, 90, 180, 365].into_iter().map(|d| {
                            let label = if d == 365 { "1y".to_string() } else { format!("{d}d") };
                            view! {
                                <button
                                    class=move || format!("px-2 py-1 text-xs rounded {}", if days.get() == d { "bg-accent text-bg font-bold" } else { "bg-border text-text" })
                                    on:click=move |_| { set_days.set(d); load_charts.dispatch(()); }
                                >{label}</button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <input type="range" min="1" max="365" prop:value=move || days.get().to_string() class="w-32 accent-accent"
                        on:input=move |e| {
                            use wasm_bindgen::JsCast;
                            let v = e.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>().value();
                            if let Ok(n) = v.parse::<i64>() { set_days.set(n); }
                        }
                        on:change=move |_| { load_charts.dispatch(()); }
                    />
                    <span class="text-text text-sm font-bold">{move || if days.get() >= 365 { "1 year".into() } else { format!("{} days", days.get()) }}</span>
                </div>
                <Show when=move || show_trends.get()>
                    <TrendsSection data=daily_data />
                </Show>
            </Show>
        </div>
    }
}

fn load_setting(key: &str, default: &str) -> String {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
        .unwrap_or_else(|| default.to_string())
}

fn save_setting(key: &str, value: &str) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(key, value);
    }
}
