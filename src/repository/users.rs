use super::Repository;
use uuid::Uuid;

/// A row from the garmin_users table
pub struct GarminUser {
    pub user_id: Uuid,
    pub garmin_username: String,
    pub encrypted_password: String,
    pub password_nonce: String,
    pub encrypted_session: Option<String>,
    pub session_nonce: Option<String>,
    pub status: String,
    pub last_sync_at: Option<f64>,
}

impl Repository {
    pub fn create_user(
        &self,
        user_id: Uuid,
        garmin_username: &str,
        encrypted_password: &str,
        password_nonce: &str,
    ) -> anyhow::Result<()> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp() as f64;
        conn.execute(
            "INSERT INTO garmin_users (user_id, garmin_username, encrypted_password, password_nonce, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'connected', ?5, ?5)
             ON CONFLICT(user_id) DO UPDATE SET
                garmin_username = excluded.garmin_username,
                encrypted_password = excluded.encrypted_password,
                password_nonce = excluded.password_nonce,
                status = 'connected',
                updated_at = excluded.updated_at",
            rusqlite::params![user_id.to_string(), garmin_username, encrypted_password, password_nonce, now],
        )?;
        Ok(())
    }

    pub fn get_user(&self, user_id: Uuid) -> anyhow::Result<Option<GarminUser>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT user_id, garmin_username, encrypted_password, password_nonce,
                    encrypted_session, session_nonce, status, last_sync_at
             FROM garmin_users WHERE user_id = ?1"
        )?;
        let result = stmt.query_row(rusqlite::params![user_id.to_string()], |row| {
            Ok(GarminUser {
                user_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or(user_id),
                garmin_username: row.get(1)?,
                encrypted_password: row.get(2)?,
                password_nonce: row.get(3)?,
                encrypted_session: row.get(4)?,
                session_nonce: row.get(5)?,
                status: row.get(6)?,
                last_sync_at: row.get(7)?,
            })
        });
        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_all_users(&self) -> anyhow::Result<Vec<GarminUser>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT user_id, garmin_username, encrypted_password, password_nonce,
                    encrypted_session, session_nonce, status, last_sync_at
             FROM garmin_users WHERE status != 'disconnected'"
        )?;
        let users = stmt.query_map([], |row| {
            Ok(GarminUser {
                user_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                garmin_username: row.get(1)?,
                encrypted_password: row.get(2)?,
                password_nonce: row.get(3)?,
                encrypted_session: row.get(4)?,
                session_nonce: row.get(5)?,
                status: row.get(6)?,
                last_sync_at: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(users)
    }

    pub fn save_session(
        &self,
        user_id: Uuid,
        encrypted_session: &str,
        session_nonce: &str,
    ) -> anyhow::Result<()> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp() as f64;
        conn.execute(
            "UPDATE garmin_users SET encrypted_session = ?1, session_nonce = ?2, status = 'connected', updated_at = ?3
             WHERE user_id = ?4",
            rusqlite::params![encrypted_session, session_nonce, now, user_id.to_string()],
        )?;
        Ok(())
    }

    pub fn update_status(&self, user_id: Uuid, status: &str) -> anyhow::Result<()> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp() as f64;
        conn.execute(
            "UPDATE garmin_users SET status = ?1, updated_at = ?2 WHERE user_id = ?3",
            rusqlite::params![status, now, user_id.to_string()],
        )?;
        Ok(())
    }

    pub fn delete_user(&self, user_id: Uuid) -> anyhow::Result<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM garmin_users WHERE user_id = ?1", rusqlite::params![user_id.to_string()])?;
        Ok(())
    }

    pub fn set_last_sync(&self, user_id: Uuid, ts: f64) -> anyhow::Result<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE garmin_users SET last_sync_at = ?1 WHERE user_id = ?2",
            rusqlite::params![ts, user_id.to_string()],
        )?;
        Ok(())
    }

    pub fn get_last_sync(&self, user_id: Uuid) -> anyhow::Result<Option<f64>> {
        let conn = self.pool.get()?;
        let result: Result<f64, _> = conn.query_row(
            "SELECT last_sync_at FROM garmin_users WHERE user_id = ?1",
            rusqlite::params![user_id.to_string()],
            |row| row.get(0),
        );
        match result {
            Ok(ts) => Ok(Some(ts)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(rusqlite::Error::InvalidColumnType(_, _, _)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
