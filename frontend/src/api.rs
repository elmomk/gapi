use crate::models::*;

pub async fn fetch_users(base_url: &str, api_key: &str) -> Result<Vec<GarminUser>, String> {
    let c = client(api_key)?;
    let resp = c.get(format!("{base_url}/api/v1/users"))
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() { return Ok(Vec::new()); }
    resp.json().await.map_err(|e| format!("{e}"))
}

fn client(api_key: &str) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .default_headers({
            let mut h = reqwest::header::HeaderMap::new();
            h.insert("X-API-Key", api_key.parse().map_err(|e| format!("{e}"))?);
            h
        })
        .build()
        .map_err(|e| format!("{e}"))
}

pub async fn fetch_vitals(base_url: &str, api_key: &str, user_id: &str) -> Result<VitalsData, String> {
    let c = client(api_key)?;
    let resp = c.get(format!("{base_url}/api/v1/users/{user_id}/vitals"))
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() { return Err(format!("HTTP {}", resp.status())); }
    resp.json().await.map_err(|e| format!("{e}"))
}

pub async fn fetch_daily_range(base_url: &str, api_key: &str, user_id: &str, days: i64) -> Result<Vec<DailyData>, String> {
    let c = client(api_key)?;
    let end = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let start = (chrono::Utc::now() - chrono::Duration::days(days)).format("%Y-%m-%d").to_string();
    let resp = c.get(format!("{base_url}/api/v1/users/{user_id}/daily"))
        .query(&[("start", &start), ("end", &end)])
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() { return Err(format!("HTTP {}", resp.status())); }
    resp.json().await.map_err(|e| format!("{e}"))
}

pub async fn trigger_sync(base_url: &str, api_key: &str, user_id: &str) -> Result<String, String> {
    let c = client(api_key)?;
    let resp = c.post(format!("{base_url}/api/v1/users/{user_id}/sync"))
        .send().await.map_err(|e| format!("{e}"))?;
    let body: serde_json::Value = resp.json().await.map_err(|e| format!("{e}"))?;
    Ok(body["message"].as_str().unwrap_or("Sync triggered").to_string())
}

// === Intraday endpoints ===

pub async fn fetch_intraday_hr(base_url: &str, api_key: &str, user_id: &str, date: &str) -> Result<Vec<IntradayPoint>, String> {
    let c = client(api_key)?;
    let resp = c.get(format!("{base_url}/api/v1/users/{user_id}/intraday/heart-rate"))
        .query(&[("date", date)])
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() { return Ok(Vec::new()); }
    let body: serde_json::Value = resp.json().await.map_err(|e| format!("{e}"))?;
    serde_json::from_value(body["points"].clone()).map_err(|e| format!("{e}"))
}

pub async fn fetch_intraday_stress(base_url: &str, api_key: &str, user_id: &str, date: &str) -> Result<Vec<StressPoint>, String> {
    let c = client(api_key)?;
    let resp = c.get(format!("{base_url}/api/v1/users/{user_id}/intraday/stress"))
        .query(&[("date", date)])
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() { return Ok(Vec::new()); }
    let body: serde_json::Value = resp.json().await.map_err(|e| format!("{e}"))?;
    serde_json::from_value(body["points"].clone()).map_err(|e| format!("{e}"))
}

pub async fn fetch_intraday_hrv(base_url: &str, api_key: &str, user_id: &str, date: &str) -> Result<Vec<HrvReading>, String> {
    let c = client(api_key)?;
    let resp = c.get(format!("{base_url}/api/v1/users/{user_id}/intraday/hrv"))
        .query(&[("date", date)])
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() { return Ok(Vec::new()); }
    let body: serde_json::Value = resp.json().await.map_err(|e| format!("{e}"))?;
    serde_json::from_value(body["points"].clone()).map_err(|e| format!("{e}"))
}

pub async fn fetch_intraday_sleep(base_url: &str, api_key: &str, user_id: &str, date: &str) -> Result<Vec<SleepEpoch>, String> {
    let c = client(api_key)?;
    let resp = c.get(format!("{base_url}/api/v1/users/{user_id}/intraday/sleep"))
        .query(&[("date", date)])
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() { return Ok(Vec::new()); }
    let body: serde_json::Value = resp.json().await.map_err(|e| format!("{e}"))?;
    serde_json::from_value(body["points"].clone()).map_err(|e| format!("{e}"))
}

pub async fn fetch_daily_extended(base_url: &str, api_key: &str, user_id: &str, days: i64) -> Result<Vec<DailyExtended>, String> {
    let c = client(api_key)?;
    let end = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let start = (chrono::Utc::now() - chrono::Duration::days(days)).format("%Y-%m-%d").to_string();
    let resp = c.get(format!("{base_url}/api/v1/users/{user_id}/daily-extended"))
        .query(&[("start", &start), ("end", &end)])
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() { return Ok(Vec::new()); }
    resp.json().await.map_err(|e| format!("{e}"))
}

pub async fn fetch_intraday_respiration(base_url: &str, api_key: &str, user_id: &str, date: &str) -> Result<Vec<IntradayPointF64>, String> {
    let c = client(api_key)?;
    let resp = c.get(format!("{base_url}/api/v1/users/{user_id}/intraday/respiration"))
        .query(&[("date", date)])
        .send().await.map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() { return Ok(Vec::new()); }
    let body: serde_json::Value = resp.json().await.map_err(|e| format!("{e}"))?;
    serde_json::from_value(body["points"].clone()).map_err(|e| format!("{e}"))
}
