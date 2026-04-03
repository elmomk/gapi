pub mod credentials;
pub mod sync;
pub mod data;
pub mod intraday;
pub mod activities;
pub mod webhooks;
pub mod health;

use std::sync::Arc;
use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
};
use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    let api = Router::new()
        .merge(credentials::routes())
        .merge(sync::routes())
        .merge(data::routes())
        .merge(intraday::routes())
        .merge(activities::routes())
        .merge(webhooks::routes())
        .layer(middleware::from_fn_with_state(state.clone(), api_key_auth));

    Router::new()
        .nest("/api/v1", api)
        .merge(health::routes())
        .with_state(state)
}

/// API key authentication middleware
async fn api_key_auth(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let api_key = req.headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) => {
            match state.repo.validate_api_key(key) {
                Ok(Some(_consumer)) => Ok(next.run(req).await),
                _ => Err(StatusCode::UNAUTHORIZED),
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
