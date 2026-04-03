use super::Repository;
use crate::domain::{DailyExtended, GpsTrackPoint};
use uuid::Uuid;

impl Repository {
    pub fn upsert_daily_extended(&self, d: &DailyExtended) -> anyhow::Result<()> {
        let conn = self.pool.get()?;
        let now = chrono::Utc::now().timestamp() as f64;
        conn.execute(
            "INSERT INTO daily_extended (
                user_id, date, fitness_age, race_5k_secs, race_10k_secs,
                race_half_secs, race_marathon_secs, hydration_intake_ml, hydration_goal_ml,
                systolic_bp, diastolic_bp, training_status_phase, acute_training_load,
                low_stress_secs, medium_stress_secs, high_stress_secs, rest_stress_secs,
                sedentary_secs, active_secs, highly_active_secs, synced_at
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21)
            ON CONFLICT (user_id, date) DO UPDATE SET
                fitness_age=COALESCE(excluded.fitness_age, daily_extended.fitness_age),
                race_5k_secs=COALESCE(excluded.race_5k_secs, daily_extended.race_5k_secs),
                race_10k_secs=COALESCE(excluded.race_10k_secs, daily_extended.race_10k_secs),
                race_half_secs=COALESCE(excluded.race_half_secs, daily_extended.race_half_secs),
                race_marathon_secs=COALESCE(excluded.race_marathon_secs, daily_extended.race_marathon_secs),
                hydration_intake_ml=COALESCE(excluded.hydration_intake_ml, daily_extended.hydration_intake_ml),
                hydration_goal_ml=COALESCE(excluded.hydration_goal_ml, daily_extended.hydration_goal_ml),
                systolic_bp=COALESCE(excluded.systolic_bp, daily_extended.systolic_bp),
                diastolic_bp=COALESCE(excluded.diastolic_bp, daily_extended.diastolic_bp),
                training_status_phase=COALESCE(excluded.training_status_phase, daily_extended.training_status_phase),
                acute_training_load=COALESCE(excluded.acute_training_load, daily_extended.acute_training_load),
                low_stress_secs=COALESCE(excluded.low_stress_secs, daily_extended.low_stress_secs),
                medium_stress_secs=COALESCE(excluded.medium_stress_secs, daily_extended.medium_stress_secs),
                high_stress_secs=COALESCE(excluded.high_stress_secs, daily_extended.high_stress_secs),
                rest_stress_secs=COALESCE(excluded.rest_stress_secs, daily_extended.rest_stress_secs),
                sedentary_secs=COALESCE(excluded.sedentary_secs, daily_extended.sedentary_secs),
                active_secs=COALESCE(excluded.active_secs, daily_extended.active_secs),
                highly_active_secs=COALESCE(excluded.highly_active_secs, daily_extended.highly_active_secs),
                synced_at=excluded.synced_at",
            rusqlite::params![
                d.user_id, d.date, d.fitness_age, d.race_5k_secs, d.race_10k_secs,
                d.race_half_secs, d.race_marathon_secs, d.hydration_intake_ml, d.hydration_goal_ml,
                d.systolic_bp, d.diastolic_bp, d.training_status_phase, d.acute_training_load,
                d.low_stress_secs, d.medium_stress_secs, d.high_stress_secs, d.rest_stress_secs,
                d.sedentary_secs, d.active_secs, d.highly_active_secs, now,
            ],
        )?;
        Ok(())
    }

    pub fn get_daily_extended(&self, user_id: Uuid, date: &str) -> anyhow::Result<Option<DailyExtended>> {
        let conn = self.pool.get()?;
        let result = conn.query_row(
            "SELECT user_id, date, fitness_age, race_5k_secs, race_10k_secs,
                    race_half_secs, race_marathon_secs, hydration_intake_ml, hydration_goal_ml,
                    systolic_bp, diastolic_bp, training_status_phase, acute_training_load,
                    low_stress_secs, medium_stress_secs, high_stress_secs, rest_stress_secs,
                    sedentary_secs, active_secs, highly_active_secs
             FROM daily_extended WHERE user_id = ?1 AND date = ?2",
            rusqlite::params![user_id.to_string(), date],
            |row| {
                Ok(DailyExtended {
                    user_id: row.get(0)?,
                    date: row.get(1)?,
                    fitness_age: row.get(2)?,
                    race_5k_secs: row.get(3)?,
                    race_10k_secs: row.get(4)?,
                    race_half_secs: row.get(5)?,
                    race_marathon_secs: row.get(6)?,
                    hydration_intake_ml: row.get(7)?,
                    hydration_goal_ml: row.get(8)?,
                    systolic_bp: row.get(9)?,
                    diastolic_bp: row.get(10)?,
                    training_status_phase: row.get(11)?,
                    acute_training_load: row.get(12)?,
                    low_stress_secs: row.get(13)?,
                    medium_stress_secs: row.get(14)?,
                    high_stress_secs: row.get(15)?,
                    rest_stress_secs: row.get(16)?,
                    sedentary_secs: row.get(17)?,
                    active_secs: row.get(18)?,
                    highly_active_secs: row.get(19)?,
                })
            },
        );
        match result {
            Ok(d) => Ok(Some(d)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_daily_extended_range(&self, user_id: Uuid, start: &str, end: &str) -> anyhow::Result<Vec<DailyExtended>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT user_id, date, fitness_age, race_5k_secs, race_10k_secs,
                    race_half_secs, race_marathon_secs, hydration_intake_ml, hydration_goal_ml,
                    systolic_bp, diastolic_bp, training_status_phase, acute_training_load,
                    low_stress_secs, medium_stress_secs, high_stress_secs, rest_stress_secs,
                    sedentary_secs, active_secs, highly_active_secs
             FROM daily_extended WHERE user_id = ?1 AND date >= ?2 AND date <= ?3 ORDER BY date ASC"
        )?;
        let rows = stmt.query_map(rusqlite::params![user_id.to_string(), start, end], |row| {
            Ok(DailyExtended {
                user_id: row.get(0)?,
                date: row.get(1)?,
                fitness_age: row.get(2)?,
                race_5k_secs: row.get(3)?,
                race_10k_secs: row.get(4)?,
                race_half_secs: row.get(5)?,
                race_marathon_secs: row.get(6)?,
                hydration_intake_ml: row.get(7)?,
                hydration_goal_ml: row.get(8)?,
                systolic_bp: row.get(9)?,
                diastolic_bp: row.get(10)?,
                training_status_phase: row.get(11)?,
                acute_training_load: row.get(12)?,
                low_stress_secs: row.get(13)?,
                medium_stress_secs: row.get(14)?,
                high_stress_secs: row.get(15)?,
                rest_stress_secs: row.get(16)?,
                sedentary_secs: row.get(17)?,
                active_secs: row.get(18)?,
                highly_active_secs: row.get(19)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // === GPS Track methods ===

    pub fn upsert_gps_track(&self, activity_id: i64, user_id: &str, date: &str, points: &[GpsTrackPoint]) -> anyhow::Result<()> {
        if points.is_empty() { return Ok(()); }
        let conn = self.pool.get()?;
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM activity_gps_tracks WHERE activity_id = ?1",
            rusqlite::params![activity_id],
        )?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO activity_gps_tracks (activity_id, ts_ms, user_id, date, lat, lon, altitude_m, speed_mps, hr, cadence, power_w)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
            )?;
            for p in points {
                stmt.execute(rusqlite::params![
                    activity_id, p.ts, user_id, date,
                    p.lat, p.lon, p.altitude_m, p.speed_mps, p.hr, p.cadence, p.power_w,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_gps_track(&self, activity_id: i64) -> anyhow::Result<Vec<GpsTrackPoint>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT ts_ms, lat, lon, altitude_m, speed_mps, hr, cadence, power_w
             FROM activity_gps_tracks WHERE activity_id = ?1 ORDER BY ts_ms"
        )?;
        let points = stmt.query_map(rusqlite::params![activity_id], |row| {
            Ok(GpsTrackPoint {
                ts: row.get(0)?,
                lat: row.get(1)?,
                lon: row.get(2)?,
                altitude_m: row.get(3)?,
                speed_mps: row.get(4)?,
                hr: row.get(5)?,
                cadence: row.get(6)?,
                power_w: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(points)
    }
}
