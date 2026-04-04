use leptos::prelude::*;
use crate::models::*;
use crate::api;

#[derive(Clone, Copy)]
pub struct AppState {
    pub api_url: RwSignal<String>,
    pub api_key: RwSignal<String>,
    pub user_id: RwSignal<String>,
    pub users: RwSignal<Vec<GarminUser>>,
    pub days: RwSignal<i64>,
    pub vitals: RwSignal<Option<VitalsData>>,
    pub daily_data: RwSignal<Vec<DailyData>>,
    pub intraday_hr: RwSignal<Vec<IntradayPoint>>,
    pub intraday_stress: RwSignal<Vec<StressPoint>>,
    pub intraday_hrv: RwSignal<Vec<HrvReading>>,
    pub intraday_sleep: RwSignal<Vec<SleepEpoch>>,
    pub intraday_resp: RwSignal<Vec<IntradayPointF64>>,
    pub extended_data: RwSignal<Vec<DailyExtended>>,
    pub sleep_target_hours: RwSignal<f64>,
    pub loading: RwSignal<bool>,
    pub status: RwSignal<(String, String)>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            api_url: RwSignal::new(crate::load_setting("garmin_api_url", "")),
            api_key: RwSignal::new(crate::load_setting("garmin_api_key", "")),
            user_id: RwSignal::new(crate::load_setting("garmin_user_id", "")),
            users: RwSignal::new(Vec::new()),
            days: RwSignal::new(30),
            vitals: RwSignal::new(None),
            daily_data: RwSignal::new(Vec::new()),
            intraday_hr: RwSignal::new(Vec::new()),
            intraday_stress: RwSignal::new(Vec::new()),
            intraday_hrv: RwSignal::new(Vec::new()),
            intraday_sleep: RwSignal::new(Vec::new()),
            intraday_resp: RwSignal::new(Vec::new()),
            extended_data: RwSignal::new(Vec::new()),
            sleep_target_hours: RwSignal::new(
                crate::load_setting("sleep_target_hours", "7.0").parse().unwrap_or(7.0)
            ),
            loading: RwSignal::new(false),
            status: RwSignal::new((String::new(), String::new())),
        }
    }

    pub async fn bootstrap(&self) {
        // Load config.json if credentials empty
        if self.api_key.get_untracked().is_empty() {
            let origin = web_sys::window().and_then(|w| w.location().origin().ok()).unwrap_or_default();
            if let Ok(resp) = reqwest::Client::new().get(format!("{}/config.json", origin)).send().await {
                if let Ok(cfg) = resp.json::<serde_json::Value>().await {
                    if let Some(u) = cfg["api_url"].as_str() { if self.api_url.get_untracked().is_empty() { self.api_url.set(u.to_string()); } }
                    if let Some(k) = cfg["api_key"].as_str() { if self.api_key.get_untracked().is_empty() { self.api_key.set(k.to_string()); } }
                    // user_id from config is fallback only
                    if let Some(id) = cfg["user_id"].as_str() { if self.user_id.get_untracked().is_empty() { self.user_id.set(id.to_string()); } }
                }
            }
        }

        // Save settings
        crate::save_setting("garmin_api_url", &self.api_url.get_untracked());
        crate::save_setting("garmin_api_key", &self.api_key.get_untracked());

        // Fetch user list
        if !self.api_key.get_untracked().is_empty() {
            if let Ok(users) = api::fetch_users(&self.api_url.get_untracked(), &self.api_key.get_untracked()).await {
                self.users.set(users.clone());
                // Auto-select first user if none selected
                let stored = self.user_id.get_untracked();
                if stored.is_empty() || !users.iter().any(|u| u.user_id == stored) {
                    if let Some(first) = users.first() {
                        self.user_id.set(first.user_id.clone());
                    }
                }
            }
        }

        crate::save_setting("garmin_user_id", &self.user_id.get_untracked());

        // Load data
        if !self.api_key.get_untracked().is_empty() && !self.user_id.get_untracked().is_empty() {
            self.load_all().await;
        }
    }

    pub async fn switch_user(&self, uid: String) {
        self.user_id.set(uid);
        crate::save_setting("garmin_user_id", &self.user_id.get_untracked());
        self.load_all().await;
    }

    pub async fn load_all(&self) {
        self.loading.set(true);
        self.status.set(("Loading...".into(), "loading".into()));

        let url = self.api_url.get_untracked();
        let key = self.api_key.get_untracked();
        let uid = self.user_id.get_untracked();
        let d = self.days.get_untracked();

        match api::fetch_vitals(&url, &key, &uid).await {
            Ok(v) => self.vitals.set(Some(v)),
            Err(e) => { self.status.set((format!("Error: {e}"), "err".into())); self.loading.set(false); return; }
        }

        match api::fetch_daily_range(&url, &key, &uid, d).await {
            Ok(data) => { let n = data.len(); self.daily_data.set(data); self.status.set((format!("Loaded {n} days"), "ok".into())); }
            Err(e) => { self.status.set((format!("Error: {e}"), "err".into())); }
        }

        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        self.intraday_hr.set(api::fetch_intraday_hr(&url, &key, &uid, &today).await.unwrap_or_default());
        self.intraday_stress.set(api::fetch_intraday_stress(&url, &key, &uid, &today).await.unwrap_or_default());
        self.intraday_hrv.set(api::fetch_intraday_hrv(&url, &key, &uid, &today).await.unwrap_or_default());
        self.intraday_sleep.set(api::fetch_intraday_sleep(&url, &key, &uid, &today).await.unwrap_or_default());
        self.intraday_resp.set(api::fetch_intraday_respiration(&url, &key, &uid, &today).await.unwrap_or_default());
        self.extended_data.set(api::fetch_daily_extended(&url, &key, &uid, d).await.unwrap_or_default());

        self.loading.set(false);
    }

    pub async fn load_daily(&self) {
        let url = self.api_url.get_untracked();
        let key = self.api_key.get_untracked();
        let uid = self.user_id.get_untracked();
        let d = self.days.get_untracked();
        if let Ok(data) = api::fetch_daily_range(&url, &key, &uid, d).await {
            self.daily_data.set(data);
        }
        self.extended_data.set(api::fetch_daily_extended(&url, &key, &uid, d).await.unwrap_or_default());
    }

    pub async fn trigger_sync(&self) {
        let url = self.api_url.get_untracked();
        let key = self.api_key.get_untracked();
        let uid = self.user_id.get_untracked();
        self.status.set(("Syncing...".into(), "loading".into()));
        match api::trigger_sync(&url, &key, &uid).await {
            Ok(msg) => self.status.set((msg, "ok".into())),
            Err(e) => self.status.set((format!("Sync failed: {e}"), "err".into())),
        }
    }
}
