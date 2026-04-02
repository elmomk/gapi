pub mod webhook;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
}

impl Event {
    pub fn new(event_type: &str, user_id: Uuid, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            user_id,
            timestamp: Utc::now(),
            payload,
        }
    }
}

/// Webhook subscription stored in DB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSubscription {
    pub id: String,
    pub consumer_name: String,
    pub url: String,
    pub secret: Option<String>,
    pub event_types: Vec<String>,
    pub active: bool,
}
