use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::post,
};
use uuid::Uuid;
use crate::state::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users/{user_id}/sync", post(trigger_sync))
}

async fn trigger_sync(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match crate::sync::perform_user_sync(&state, user_id).await {
        Ok(msg) => Ok(Json(serde_json::json!({ "status": "ok", "message": msg }))),
        Err(e) => Ok(Json(serde_json::json!({ "status": "error", "message": e.to_string() }))),
    }
}
