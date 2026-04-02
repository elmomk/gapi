mod config;
mod db;
mod domain;
mod events;
mod garmin;
mod handlers;
mod repository;
mod state;
mod sync;
mod vault;

use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = config::AppConfig::from_env();
    let bind_addr = format!("{}:{}", config.host, config.port);

    let pool = db::init_pool(&config.database_path);

    let state = state::AppState::new(config, pool);

    // Seed API keys from env if provided (comma-separated "key:consumer_name" pairs)
    if let Ok(keys_str) = std::env::var("API_KEYS") {
        for entry in keys_str.split(',') {
            let entry = entry.trim();
            if let Some((key, name)) = entry.split_once(':')
                && let Err(e) = state.repo.create_api_key(key.trim(), name.trim()) {
                    tracing::warn!("Failed to seed API key for {}: {}", name, e);
                }
        }
    }

    // Spawn background sync worker
    let bg_state = Arc::clone(&state);
    tokio::spawn(sync::background_sync_loop(bg_state));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = handlers::router(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&bind_addr).await
        .expect("Failed to bind");
    tracing::info!("garmin_api listening on {}", bind_addr);

    axum::serve(listener, app).await
        .expect("Server error");
}
