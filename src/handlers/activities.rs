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
pub struct DateRangeQuery {
    start: Option<String>,
    end: Option<String>,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users/{user_id}/activities", get(list_activities))
}

async fn list_activities(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<DateRangeQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();
    let start = q.start.unwrap_or_else(|| {
        (chrono::Utc::now().date_naive() - chrono::Duration::days(7)).format("%Y-%m-%d").to_string()
    });
    let end = q.end.unwrap_or(today);

    match state.repo.get_daily_range(user_id, &start, &end) {
        Ok(days) => {
            let mut activities = Vec::new();
            for day in &days {
                if let Some(ref json) = day.activities_json {
                    if let Ok(acts) = serde_json::from_str::<Vec<serde_json::Value>>(json) {
                        for mut act in acts {
                            act["date"] = serde_json::json!(day.date.format("%Y-%m-%d").to_string());
                            activities.push(act);
                        }
                    }
                }
            }
            Ok(Json(serde_json::json!(activities)))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
