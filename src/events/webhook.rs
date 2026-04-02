use super::{Event, WebhookSubscription};
use crate::repository::Repository;

pub struct WebhookDispatcher {
    client: reqwest::Client,
}

impl WebhookDispatcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    /// Dispatch an event to all matching webhook subscribers
    pub async fn dispatch(&self, repo: &Repository, event: Event) {
        let subscriptions = match repo.get_active_webhooks() {
            Ok(subs) => subs,
            Err(e) => {
                tracing::error!("Failed to get webhook subscriptions: {}", e);
                return;
            }
        };

        for sub in subscriptions {
            if !sub.event_types.contains(&event.event_type) && !sub.event_types.contains(&"*".to_string()) {
                continue;
            }
            let client = self.client.clone();
            let event_clone = event.clone();
            let sub_clone = sub.clone();
            tokio::spawn(async move {
                deliver_webhook(&client, &sub_clone, &event_clone).await;
            });
        }
    }
}

/// Deliver a webhook with retries (3 attempts, exponential backoff: 1s, 4s, 16s)
async fn deliver_webhook(client: &reqwest::Client, sub: &WebhookSubscription, event: &Event) {
    let body = match serde_json::to_string(event) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("Failed to serialize event: {}", e);
            return;
        }
    };

    for attempt in 0..3u32 {
        if attempt > 0 {
            let delay = std::time::Duration::from_secs(4u64.pow(attempt - 1));
            tokio::time::sleep(delay).await;
        }

        let mut req = client
            .post(&sub.url)
            .header("Content-Type", "application/json")
            .header("X-Event-Type", &event.event_type)
            .header("X-Event-Id", event.id.to_string());

        // HMAC signature if secret is configured
        if let Some(ref secret) = sub.secret {
            use hmac::{Hmac, Mac};
            use sha2::Sha256;
            use base64::prelude::*;

            type HmacSha256 = Hmac<Sha256>;
            if let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) {
                mac.update(body.as_bytes());
                let signature = BASE64_STANDARD.encode(mac.finalize().into_bytes());
                req = req.header("X-Webhook-Signature", signature);
            }
        }

        match req.body(body.clone()).send().await {
            Ok(resp) if resp.status().is_success() => {
                tracing::debug!(
                    "Webhook delivered to {} (event={}, attempt={})",
                    sub.url, event.event_type, attempt + 1
                );
                return;
            }
            Ok(resp) => {
                tracing::warn!(
                    "Webhook {} returned {} (event={}, attempt={})",
                    sub.url, resp.status(), event.event_type, attempt + 1
                );
            }
            Err(e) => {
                tracing::warn!(
                    "Webhook {} failed (event={}, attempt={}): {}",
                    sub.url, event.event_type, attempt + 1, e
                );
            }
        }
    }
    tracing::error!("Webhook delivery to {} failed after 3 attempts (event={})", sub.url, event.event_type);
}
