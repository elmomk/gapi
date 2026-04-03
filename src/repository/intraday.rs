use super::Repository;
use crate::domain::*;
use uuid::Uuid;

/// Helper: batch insert into a table within a transaction.
/// Deletes existing data for the date first, then inserts all points.
fn batch_upsert(
    conn: &rusqlite::Connection,
    delete_sql: &str,
    insert_sql: &str,
    user_id: &str,
    date: &str,
    bind_fn: &dyn Fn(&mut rusqlite::Statement, &str, &str) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(delete_sql, rusqlite::params![user_id, date])?;
    {
        let mut stmt = tx.prepare(insert_sql)?;
        bind_fn(&mut stmt, user_id, date)?;
    } // stmt dropped here, releasing borrow on tx
    tx.commit()?;
    Ok(())
}

impl Repository {
    pub fn upsert_intraday_hr(&self, user_id: Uuid, date: &str, points: &[IntradayPoint]) -> anyhow::Result<()> {
        if points.is_empty() { return Ok(()); }
        let conn = self.pool.get()?;
        let uid = user_id.to_string();
        batch_upsert(
            &conn,
            "DELETE FROM intraday_heart_rate WHERE user_id = ?1 AND date = ?2",
            "INSERT INTO intraday_heart_rate (user_id, date, ts_ms, value) VALUES (?1, ?2, ?3, ?4)",
            &uid, date,
            &|stmt, uid, date| {
                for p in points {
                    stmt.execute(rusqlite::params![uid, date, p.ts, p.value])?;
                }
                Ok(())
            },
        )
    }

    pub fn upsert_intraday_stress(&self, user_id: Uuid, date: &str, points: &[StressPoint]) -> anyhow::Result<()> {
        if points.is_empty() { return Ok(()); }
        let conn = self.pool.get()?;
        let uid = user_id.to_string();
        batch_upsert(
            &conn,
            "DELETE FROM intraday_stress WHERE user_id = ?1 AND date = ?2",
            "INSERT INTO intraday_stress (user_id, date, ts_ms, stress, body_battery) VALUES (?1, ?2, ?3, ?4, ?5)",
            &uid, date,
            &|stmt, uid, date| {
                for p in points {
                    stmt.execute(rusqlite::params![uid, date, p.ts, p.stress, p.body_battery])?;
                }
                Ok(())
            },
        )
    }

    pub fn upsert_intraday_steps(&self, user_id: Uuid, date: &str, points: &[IntradayPoint]) -> anyhow::Result<()> {
        if points.is_empty() { return Ok(()); }
        let conn = self.pool.get()?;
        let uid = user_id.to_string();
        batch_upsert(
            &conn,
            "DELETE FROM intraday_steps WHERE user_id = ?1 AND date = ?2",
            "INSERT INTO intraday_steps (user_id, date, ts_ms, steps) VALUES (?1, ?2, ?3, ?4)",
            &uid, date,
            &|stmt, uid, date| {
                for p in points {
                    stmt.execute(rusqlite::params![uid, date, p.ts, p.value])?;
                }
                Ok(())
            },
        )
    }

    pub fn upsert_intraday_respiration(&self, user_id: Uuid, date: &str, points: &[IntradayPointF64]) -> anyhow::Result<()> {
        if points.is_empty() { return Ok(()); }
        let conn = self.pool.get()?;
        let uid = user_id.to_string();
        batch_upsert(
            &conn,
            "DELETE FROM intraday_respiration WHERE user_id = ?1 AND date = ?2",
            "INSERT INTO intraday_respiration (user_id, date, ts_ms, value) VALUES (?1, ?2, ?3, ?4)",
            &uid, date,
            &|stmt, uid, date| {
                for p in points {
                    stmt.execute(rusqlite::params![uid, date, p.ts, p.value])?;
                }
                Ok(())
            },
        )
    }

    pub fn upsert_intraday_hrv(&self, user_id: Uuid, date: &str, points: &[HrvReading]) -> anyhow::Result<()> {
        if points.is_empty() { return Ok(()); }
        let conn = self.pool.get()?;
        let uid = user_id.to_string();
        batch_upsert(
            &conn,
            "DELETE FROM intraday_hrv WHERE user_id = ?1 AND date = ?2",
            "INSERT INTO intraday_hrv (user_id, date, ts_ms, hrv_value) VALUES (?1, ?2, ?3, ?4)",
            &uid, date,
            &|stmt, uid, date| {
                for p in points {
                    stmt.execute(rusqlite::params![uid, date, p.ts, p.hrv_value])?;
                }
                Ok(())
            },
        )
    }

    pub fn upsert_intraday_sleep(&self, user_id: Uuid, date: &str, points: &[SleepEpoch]) -> anyhow::Result<()> {
        if points.is_empty() { return Ok(()); }
        let conn = self.pool.get()?;
        let uid = user_id.to_string();
        batch_upsert(
            &conn,
            "DELETE FROM intraday_sleep WHERE user_id = ?1 AND date = ?2",
            "INSERT INTO intraday_sleep (user_id, date, ts_ms, stage, hr, spo2, respiration, movement) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            &uid, date,
            &|stmt, uid, date| {
                for p in points {
                    stmt.execute(rusqlite::params![uid, date, p.ts, p.stage, p.hr, p.spo2, p.respiration, p.movement])?;
                }
                Ok(())
            },
        )
    }

    // Query methods

    pub fn get_intraday_hr(&self, user_id: Uuid, date: &str) -> anyhow::Result<Vec<IntradayPoint>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT ts_ms, value FROM intraday_heart_rate WHERE user_id = ?1 AND date = ?2 ORDER BY ts_ms")?;
        let points = stmt.query_map(rusqlite::params![user_id.to_string(), date], |row| {
            Ok(IntradayPoint { ts: row.get(0)?, value: row.get(1)? })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(points)
    }

    pub fn get_intraday_stress(&self, user_id: Uuid, date: &str) -> anyhow::Result<Vec<StressPoint>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT ts_ms, stress, body_battery FROM intraday_stress WHERE user_id = ?1 AND date = ?2 ORDER BY ts_ms")?;
        let points = stmt.query_map(rusqlite::params![user_id.to_string(), date], |row| {
            Ok(StressPoint { ts: row.get(0)?, stress: row.get(1)?, body_battery: row.get(2)? })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(points)
    }

    pub fn get_intraday_steps(&self, user_id: Uuid, date: &str) -> anyhow::Result<Vec<IntradayPoint>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT ts_ms, steps FROM intraday_steps WHERE user_id = ?1 AND date = ?2 ORDER BY ts_ms")?;
        let points = stmt.query_map(rusqlite::params![user_id.to_string(), date], |row| {
            Ok(IntradayPoint { ts: row.get(0)?, value: row.get(1)? })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(points)
    }

    pub fn get_intraday_respiration(&self, user_id: Uuid, date: &str) -> anyhow::Result<Vec<IntradayPointF64>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT ts_ms, value FROM intraday_respiration WHERE user_id = ?1 AND date = ?2 ORDER BY ts_ms")?;
        let points = stmt.query_map(rusqlite::params![user_id.to_string(), date], |row| {
            Ok(IntradayPointF64 { ts: row.get(0)?, value: row.get(1)? })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(points)
    }

    pub fn get_intraday_hrv(&self, user_id: Uuid, date: &str) -> anyhow::Result<Vec<HrvReading>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT ts_ms, hrv_value FROM intraday_hrv WHERE user_id = ?1 AND date = ?2 ORDER BY ts_ms")?;
        let points = stmt.query_map(rusqlite::params![user_id.to_string(), date], |row| {
            Ok(HrvReading { ts: row.get(0)?, hrv_value: row.get(1)? })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(points)
    }

    pub fn get_intraday_sleep(&self, user_id: Uuid, date: &str) -> anyhow::Result<Vec<SleepEpoch>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT ts_ms, stage, hr, spo2, respiration, movement FROM intraday_sleep WHERE user_id = ?1 AND date = ?2 ORDER BY ts_ms")?;
        let points = stmt.query_map(rusqlite::params![user_id.to_string(), date], |row| {
            Ok(SleepEpoch {
                ts: row.get(0)?, stage: row.get(1)?, hr: row.get(2)?,
                spo2: row.get(3)?, respiration: row.get(4)?, movement: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(points)
    }

    pub fn cleanup_old_intraday(&self, user_id: Uuid, before_date: &str) -> anyhow::Result<()> {
        let conn = self.pool.get()?;
        for table in &["intraday_heart_rate", "intraday_stress", "intraday_steps",
                       "intraday_respiration", "intraday_hrv", "intraday_sleep"] {
            conn.execute(
                &format!("DELETE FROM {} WHERE user_id = ?1 AND date < ?2", table),
                rusqlite::params![user_id.to_string(), before_date],
            )?;
        }
        Ok(())
    }
}
