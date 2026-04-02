use super::Repository;
use crate::events::WebhookSubscription;

impl Repository {
    pub fn create_webhook(
        &self,
        consumer_name: &str,
        url: &str,
        secret: Option<&str>,
        event_types: &[String],
    ) -> anyhow::Result<String> {
        let conn = self.pool.get()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp() as f64;
        let event_types_json = serde_json::to_string(event_types)?;
        conn.execute(
            "INSERT INTO webhook_subscriptions (id, consumer_name, url, secret, event_types, active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
            rusqlite::params![id, consumer_name, url, secret, event_types_json, now],
        )?;
        Ok(id)
    }

    pub fn get_active_webhooks(&self) -> anyhow::Result<Vec<WebhookSubscription>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, consumer_name, url, secret, event_types, active FROM webhook_subscriptions WHERE active = 1"
        )?;
        let subs = stmt.query_map([], |row| {
            let event_types_json: String = row.get(4)?;
            let event_types: Vec<String> = serde_json::from_str(&event_types_json).unwrap_or_default();
            Ok(WebhookSubscription {
                id: row.get(0)?,
                consumer_name: row.get(1)?,
                url: row.get(2)?,
                secret: row.get(3)?,
                event_types,
                active: row.get::<_, i32>(5)? == 1,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(subs)
    }

    pub fn list_webhooks(&self) -> anyhow::Result<Vec<WebhookSubscription>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, consumer_name, url, secret, event_types, active FROM webhook_subscriptions"
        )?;
        let subs = stmt.query_map([], |row| {
            let event_types_json: String = row.get(4)?;
            let event_types: Vec<String> = serde_json::from_str(&event_types_json).unwrap_or_default();
            Ok(WebhookSubscription {
                id: row.get(0)?,
                consumer_name: row.get(1)?,
                url: row.get(2)?,
                secret: row.get(3)?,
                event_types,
                active: row.get::<_, i32>(5)? == 1,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(subs)
    }

    pub fn delete_webhook(&self, id: &str) -> anyhow::Result<bool> {
        let conn = self.pool.get()?;
        let rows = conn.execute(
            "DELETE FROM webhook_subscriptions WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(rows > 0)
    }

    pub fn validate_api_key(&self, key: &str) -> anyhow::Result<Option<String>> {
        use sha2::{Sha256, Digest};
        let key_hash = format!("{:x}", Sha256::digest(key.as_bytes()));
        let conn = self.pool.get()?;
        let result = conn.query_row(
            "SELECT consumer_name FROM api_keys WHERE key_hash = ?1",
            rusqlite::params![key_hash],
            |row| row.get(0),
        );
        match result {
            Ok(name) => Ok(Some(name)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_api_key(&self, key: &str, consumer_name: &str) -> anyhow::Result<()> {
        use sha2::{Sha256, Digest};
        let key_hash = format!("{:x}", Sha256::digest(key.as_bytes()));
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp() as f64;
        conn.execute(
            "INSERT OR REPLACE INTO api_keys (key_hash, consumer_name, created_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![key_hash, consumer_name, now],
        )?;
        Ok(())
    }
}
