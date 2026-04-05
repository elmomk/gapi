use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;
use crate::domain::VitalsResponse;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct DailyQuery {
    date: Option<String>,
    start: Option<String>,
    end: Option<String>,
}

#[derive(Deserialize)]
pub struct BaselineQuery {
    date: Option<String>,
    days: Option<i64>,
}

#[derive(Deserialize)]
pub struct VitalsQuery {
    sleep_target: Option<f64>,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users/{user_id}/daily", get(get_daily))
        .route("/users/{user_id}/baseline", get(get_baseline))
        .route("/users/{user_id}/vitals", get(get_vitals))
}

async fn get_daily(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<DailyQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(date) = q.date {
        match state.repo.get_daily(user_id, &date) {
            Ok(Some(data)) => Ok(Json(serde_json::to_value(data).unwrap_or_default())),
            Ok(None) => Err(StatusCode::NOT_FOUND),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else if let (Some(start), Some(end)) = (q.start, q.end) {
        match state.repo.get_daily_range(user_id, &start, &end) {
            Ok(data) => Ok(Json(serde_json::to_value(data).unwrap_or_default())),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        // Default to today
        let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();
        match state.repo.get_daily(user_id, &today) {
            Ok(Some(data)) => Ok(Json(serde_json::to_value(data).unwrap_or_default())),
            Ok(None) => Err(StatusCode::NOT_FOUND),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

async fn get_baseline(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<BaselineQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let date = q.date.unwrap_or_else(|| chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string());
    let days = q.days.unwrap_or(7);
    match state.repo.get_baseline(user_id, &date, days) {
        Ok(baseline) => Ok(Json(serde_json::to_value(baseline).unwrap_or_default())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_vitals(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<VitalsQuery>,
) -> Result<Json<VitalsResponse>, StatusCode> {
    let sleep_target = q.sleep_target.unwrap_or(7.0);
    let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();

    let daily = state.repo.get_daily(user_id, &today)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let baseline = state.repo.get_baseline(user_id, &today, 7)
        .ok();

    let (data_hrv, data_status, data_rhr, data_stress, data_bb_high, data_bb_low,
         data_sleep_score, data_sleep_secs, data_readiness, data_steps) = match &daily {
        Some(d) => (
            d.hrv_last_night, d.hrv_status.clone(), d.resting_heart_rate,
            d.avg_stress, d.body_battery_high, d.body_battery_low,
            d.sleep_score, d.sleep_duration_secs, d.training_readiness, d.steps,
        ),
        None => (None, None, None, None, None, None, None, None, None, None),
    };

    let sleep_hours = data_sleep_secs.map(|s| s as f64 / 3600.0);

    let (bl_hrv, bl_rhr, bl_stress, bl_battery, bl_sleep) = match &baseline {
        Some(b) => (b.avg_hrv, b.avg_rhr, b.avg_stress, b.avg_body_battery, b.avg_sleep_score),
        None => (None, None, None, None, None),
    };

    // Compute 14-day sleep debt
    let sleep_debt_hours = {
        let today = chrono::Utc::now().date_naive();
        let start = (today - chrono::Duration::days(14)).format("%Y-%m-%d").to_string();
        let yesterday = (today - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
        state.repo.get_daily_range(user_id, &start, &yesterday)
            .ok()
            .map(|days| {
                let target_secs = sleep_target * 3600.0;
                let debt: f64 = days.iter()
                    .filter(|d| d.sleep_duration_secs.is_some())
                    .map(|d| target_secs - d.sleep_duration_secs.unwrap() as f64)
                    .sum();
                debt / 3600.0
            })
    };

    Ok(Json(VitalsResponse {
        hrv_last_night: data_hrv,
        hrv_status: data_status,
        resting_heart_rate: data_rhr,
        avg_stress: data_stress,
        body_battery_high: data_bb_high,
        body_battery_low: data_bb_low,
        sleep_score: data_sleep_score,
        sleep_hours,
        training_readiness: data_readiness,
        steps: data_steps,
        baseline_hrv: bl_hrv,
        baseline_rhr: bl_rhr,
        baseline_stress: bl_stress,
        baseline_battery: bl_battery,
        baseline_sleep: bl_sleep,
        sleep_debt_hours,
    }))
}
