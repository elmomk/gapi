use std::sync::Arc;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{post, get},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::state::AppState;
use crate::garmin;
use crate::events;

#[derive(Deserialize)]
pub struct CredentialsRequest {
    garmin_username: String,
    garmin_password: String,
}

#[derive(Deserialize)]
pub struct MfaRequest {
    mfa_code: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub user_id: String,
    pub status: String,
    pub garmin_username: Option<String>,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users/{user_id}/credentials", post(create_credentials).delete(delete_credentials))
        .route("/users/{user_id}/mfa", post(submit_mfa))
        .route("/users/{user_id}/status", get(get_status))
}

async fn create_credentials(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<CredentialsRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Encrypt password
    let (enc_pass, nonce) = state.vault.encrypt(&body.garmin_password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save user
    state.repo.create_user(user_id, &body.garmin_username, &enc_pass, &nonce)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Attempt login immediately
    match garmin::garmin_login(&body.garmin_username, &body.garmin_password).await {
        garmin::LoginResult::Success(session) => {
            let session_json = serde_json::to_string(&session).unwrap_or_default();
            let (enc, n) = state.vault.encrypt(&session_json)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let _ = state.repo.save_session(user_id, &enc, &n);

            let event = events::Event::new("credentials_updated", user_id, serde_json::json!({ "status": "connected" }));
            state.webhook_dispatcher.dispatch(&state.repo, event).await;

            Ok(Json(serde_json::json!({ "status": "connected", "message": "Garmin login successful" })))
        }
        garmin::LoginResult::MfaRequired { .. } => {
            let _ = state.repo.update_status(user_id, "mfa_required");
            Ok(Json(serde_json::json!({ "status": "mfa_required", "message": "MFA code needed. POST /api/v1/users/{user_id}/mfa" })))
        }
        garmin::LoginResult::Error(msg) => {
            Ok(Json(serde_json::json!({ "status": "error", "message": msg })))
        }
    }
}

async fn submit_mfa(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<MfaRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = state.repo.get_user(user_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let pass = state.vault.decrypt(&user.encrypted_password, &user.password_nonce)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match garmin::garmin_submit_mfa(&user.garmin_username, &pass, &body.mfa_code).await {
        garmin::LoginResult::Success(session) => {
            let session_json = serde_json::to_string(&session).unwrap_or_default();
            let (enc, n) = state.vault.encrypt(&session_json)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let _ = state.repo.save_session(user_id, &enc, &n);

            let event = events::Event::new("credentials_updated", user_id, serde_json::json!({ "status": "connected" }));
            state.webhook_dispatcher.dispatch(&state.repo, event).await;

            Ok(Json(serde_json::json!({ "status": "connected", "message": "MFA verified, login successful" })))
        }
        garmin::LoginResult::MfaRequired { .. } => {
            Ok(Json(serde_json::json!({ "status": "mfa_required", "message": "MFA still required (wrong code?)" })))
        }
        garmin::LoginResult::Error(msg) => {
            Ok(Json(serde_json::json!({ "status": "error", "message": msg })))
        }
    }
}

async fn delete_credentials(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> StatusCode {
    match state.repo.delete_user(user_id) {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn get_status(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<StatusResponse>, StatusCode> {
    match state.repo.get_user(user_id) {
        Ok(Some(user)) => Ok(Json(StatusResponse {
            user_id: user.user_id.to_string(),
            status: user.status,
            garmin_username: Some(user.garmin_username),
        })),
        Ok(None) => Ok(Json(StatusResponse {
            user_id: user_id.to_string(),
            status: "disconnected".to_string(),
            garmin_username: None,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
