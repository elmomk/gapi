use std::sync::Arc;

use crate::config::AppConfig;
use crate::db::DbPool;
use crate::events::webhook::WebhookDispatcher;
use crate::repository::Repository;
use crate::vault::Vault;

pub struct AppState {
    pub config: AppConfig,
    pub repo: Repository,
    pub vault: Vault,
    pub http_client: reqwest::Client,
    pub webhook_dispatcher: WebhookDispatcher,
}

impl AppState {
    pub fn new(config: AppConfig, pool: DbPool) -> Arc<Self> {
        let vault = Vault::new(&config.master_key);
        let http_client = reqwest::Client::builder()
            .cookie_store(true)
            .user_agent("com.garmin.android.apps.connectmobile")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let webhook_dispatcher = WebhookDispatcher::new();

        Arc::new(Self {
            config,
            repo: Repository::new(pool),
            vault,
            http_client,
            webhook_dispatcher,
        })
    }
}
