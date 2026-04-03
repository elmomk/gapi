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
use crate::state::AppState;

#[derive(Deserialize)]
pub struct DateQuery {
    date: Option<String>,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users/{user_id}/intraday/heart-rate", get(get_hr))
        .route("/users/{user_id}/intraday/stress", get(get_stress))
        .route("/users/{user_id}/intraday/body-battery", get(get_body_battery))
        .route("/users/{user_id}/intraday/steps", get(get_steps))
        .route("/users/{user_id}/intraday/respiration", get(get_respiration))
        .route("/users/{user_id}/intraday/hrv", get(get_hrv))
        .route("/users/{user_id}/intraday/sleep", get(get_sleep))
        .route("/users/{user_id}/daily-extended", get(get_extended))
}

fn today() -> String {
    chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string()
}

async fn get_hr(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<DateQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let date = q.date.unwrap_or_else(today);
    match state.repo.get_intraday_hr(user_id, &date) {
        Ok(points) => Ok(Json(serde_json::json!({ "date": date, "points": points }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_stress(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<DateQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let date = q.date.unwrap_or_else(today);
    match state.repo.get_intraday_stress(user_id, &date) {
        Ok(points) => Ok(Json(serde_json::json!({ "date": date, "points": points }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_body_battery(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<DateQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let date = q.date.unwrap_or_else(today);
    // Body battery is stored in the stress table
    match state.repo.get_intraday_stress(user_id, &date) {
        Ok(points) => {
            let bb_points: Vec<serde_json::Value> = points.iter()
                .filter_map(|p| p.body_battery.map(|bb| serde_json::json!({ "ts": p.ts, "value": bb })))
                .collect();
            Ok(Json(serde_json::json!({ "date": date, "points": bb_points })))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_steps(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<DateQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let date = q.date.unwrap_or_else(today);
    match state.repo.get_intraday_steps(user_id, &date) {
        Ok(points) => Ok(Json(serde_json::json!({ "date": date, "points": points }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_respiration(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<DateQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let date = q.date.unwrap_or_else(today);
    match state.repo.get_intraday_respiration(user_id, &date) {
        Ok(points) => Ok(Json(serde_json::json!({ "date": date, "points": points }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_hrv(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<DateQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let date = q.date.unwrap_or_else(today);
    match state.repo.get_intraday_hrv(user_id, &date) {
        Ok(points) => Ok(Json(serde_json::json!({ "date": date, "points": points }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_sleep(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<DateQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let date = q.date.unwrap_or_else(today);
    match state.repo.get_intraday_sleep(user_id, &date) {
        Ok(points) => Ok(Json(serde_json::json!({ "date": date, "points": points }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct ExtendedQuery {
    date: Option<String>,
    start: Option<String>,
    end: Option<String>,
}

async fn get_extended(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<ExtendedQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let (Some(start), Some(end)) = (q.start, q.end) {
        match state.repo.get_daily_extended_range(user_id, &start, &end) {
            Ok(data) => Ok(Json(serde_json::to_value(data).unwrap_or_default())),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        let date = q.date.unwrap_or_else(today);
        match state.repo.get_daily_extended(user_id, &date) {
            Ok(Some(data)) => Ok(Json(serde_json::to_value(data).unwrap_or_default())),
            Ok(None) => Err(StatusCode::NOT_FOUND),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}
