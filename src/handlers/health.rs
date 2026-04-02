use std::sync::Arc;
use axum::{Router, routing::get};
use crate::state::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_check))
}

async fn health_check() -> &'static str {
    "ok"
}
