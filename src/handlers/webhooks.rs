use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{post, delete},
};
use serde::Deserialize;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateWebhookRequest {
    consumer_name: String,
    url: String,
    secret: Option<String>,
    event_types: Vec<String>,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/webhooks", post(create_webhook).get(list_webhooks))
        .route("/webhooks/{id}", delete(delete_webhook))
}

async fn create_webhook(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateWebhookRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.repo.create_webhook(&body.consumer_name, &body.url, body.secret.as_deref(), &body.event_types) {
        Ok(id) => Ok(Json(serde_json::json!({ "id": id, "status": "created" }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_webhooks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.repo.list_webhooks() {
        Ok(subs) => Ok(Json(serde_json::to_value(subs).unwrap_or_default())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_webhook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> StatusCode {
    match state.repo.delete_webhook(&id) {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
