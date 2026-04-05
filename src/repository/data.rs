use super::Repository;
use crate::domain::{Baseline, GarminDailyData};
use uuid::Uuid;

impl Repository {
    pub fn upsert_garmin_daily(&self, d: &GarminDailyData) -> anyhow::Result<()> {
        let conn = self.pool.get()?;
        let synced_at = d.synced_at.timestamp() as f64;
        conn.execute(
            "INSERT INTO garmin_daily_data (
                user_id, date, steps, distance_meters, active_calories, total_calories,
                floors_climbed, intensity_minutes, resting_heart_rate, max_heart_rate,
                min_heart_rate, avg_heart_rate, hrv_weekly_avg, hrv_last_night, hrv_status,
                sleep_score, sleep_duration_secs, deep_sleep_secs, light_sleep_secs,
                rem_sleep_secs, awake_secs, avg_stress, max_stress,
                body_battery_high, body_battery_low, body_battery_drain, body_battery_charge,
                weight_grams, bmi, body_fat_pct, muscle_mass_grams,
                avg_spo2, lowest_spo2, avg_respiration,
                training_readiness, training_load, vo2_max,
                activities_count, activities_json,
                sleep_restless_moments, sleep_avg_overnight_hr, skin_temp_overnight,
                sleep_score_feedback, training_readiness_feedback,
                synced_at
            ) VALUES (
                ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,
                ?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32,?33,?34,?35,?36,?37,?38,?39,
                ?40,?41,?42,?43,?44,?45
            ) ON CONFLICT (user_id, date) DO UPDATE SET
                steps=COALESCE(excluded.steps, garmin_daily_data.steps),
                distance_meters=COALESCE(excluded.distance_meters, garmin_daily_data.distance_meters),
                active_calories=COALESCE(excluded.active_calories, garmin_daily_data.active_calories),
                total_calories=COALESCE(excluded.total_calories, garmin_daily_data.total_calories),
                floors_climbed=COALESCE(excluded.floors_climbed, garmin_daily_data.floors_climbed),
                intensity_minutes=COALESCE(excluded.intensity_minutes, garmin_daily_data.intensity_minutes),
                resting_heart_rate=COALESCE(excluded.resting_heart_rate, garmin_daily_data.resting_heart_rate),
                max_heart_rate=COALESCE(excluded.max_heart_rate, garmin_daily_data.max_heart_rate),
                min_heart_rate=COALESCE(excluded.min_heart_rate, garmin_daily_data.min_heart_rate),
                avg_heart_rate=COALESCE(excluded.avg_heart_rate, garmin_daily_data.avg_heart_rate),
                hrv_weekly_avg=COALESCE(excluded.hrv_weekly_avg, garmin_daily_data.hrv_weekly_avg),
                hrv_last_night=COALESCE(excluded.hrv_last_night, garmin_daily_data.hrv_last_night),
                hrv_status=COALESCE(excluded.hrv_status, garmin_daily_data.hrv_status),
                sleep_score=COALESCE(excluded.sleep_score, garmin_daily_data.sleep_score),
                sleep_duration_secs=COALESCE(excluded.sleep_duration_secs, garmin_daily_data.sleep_duration_secs),
                deep_sleep_secs=COALESCE(excluded.deep_sleep_secs, garmin_daily_data.deep_sleep_secs),
                light_sleep_secs=COALESCE(excluded.light_sleep_secs, garmin_daily_data.light_sleep_secs),
                rem_sleep_secs=COALESCE(excluded.rem_sleep_secs, garmin_daily_data.rem_sleep_secs),
                awake_secs=COALESCE(excluded.awake_secs, garmin_daily_data.awake_secs),
                avg_stress=COALESCE(excluded.avg_stress, garmin_daily_data.avg_stress),
                max_stress=COALESCE(excluded.max_stress, garmin_daily_data.max_stress),
                body_battery_high=COALESCE(excluded.body_battery_high, garmin_daily_data.body_battery_high),
                body_battery_low=COALESCE(excluded.body_battery_low, garmin_daily_data.body_battery_low),
                body_battery_drain=COALESCE(excluded.body_battery_drain, garmin_daily_data.body_battery_drain),
                body_battery_charge=COALESCE(excluded.body_battery_charge, garmin_daily_data.body_battery_charge),
                weight_grams=COALESCE(excluded.weight_grams, garmin_daily_data.weight_grams),
                bmi=COALESCE(excluded.bmi, garmin_daily_data.bmi),
                body_fat_pct=COALESCE(excluded.body_fat_pct, garmin_daily_data.body_fat_pct),
                muscle_mass_grams=COALESCE(excluded.muscle_mass_grams, garmin_daily_data.muscle_mass_grams),
                avg_spo2=COALESCE(excluded.avg_spo2, garmin_daily_data.avg_spo2),
                lowest_spo2=COALESCE(excluded.lowest_spo2, garmin_daily_data.lowest_spo2),
                avg_respiration=COALESCE(excluded.avg_respiration, garmin_daily_data.avg_respiration),
                training_readiness=COALESCE(excluded.training_readiness, garmin_daily_data.training_readiness),
                training_load=COALESCE(excluded.training_load, garmin_daily_data.training_load),
                vo2_max=COALESCE(excluded.vo2_max, garmin_daily_data.vo2_max),
                activities_count=COALESCE(excluded.activities_count, garmin_daily_data.activities_count),
                activities_json=COALESCE(excluded.activities_json, garmin_daily_data.activities_json),
                sleep_restless_moments=COALESCE(excluded.sleep_restless_moments, garmin_daily_data.sleep_restless_moments),
                sleep_avg_overnight_hr=COALESCE(excluded.sleep_avg_overnight_hr, garmin_daily_data.sleep_avg_overnight_hr),
                skin_temp_overnight=COALESCE(excluded.skin_temp_overnight, garmin_daily_data.skin_temp_overnight),
                sleep_score_feedback=COALESCE(excluded.sleep_score_feedback, garmin_daily_data.sleep_score_feedback),
                training_readiness_feedback=COALESCE(excluded.training_readiness_feedback, garmin_daily_data.training_readiness_feedback),
                synced_at=excluded.synced_at",
            rusqlite::params![
                d.user_id.to_string(), d.date.format("%Y-%m-%d").to_string(),
                d.steps, d.distance_meters, d.active_calories, d.total_calories,
                d.floors_climbed, d.intensity_minutes,
                d.resting_heart_rate, d.max_heart_rate, d.min_heart_rate, d.avg_heart_rate,
                d.hrv_weekly_avg, d.hrv_last_night, d.hrv_status,
                d.sleep_score, d.sleep_duration_secs, d.deep_sleep_secs,
                d.light_sleep_secs, d.rem_sleep_secs, d.awake_secs,
                d.avg_stress, d.max_stress,
                d.body_battery_high, d.body_battery_low, d.body_battery_drain, d.body_battery_charge,
                d.weight_grams, d.bmi, d.body_fat_pct, d.muscle_mass_grams,
                d.avg_spo2, d.lowest_spo2, d.avg_respiration,
                d.training_readiness, d.training_load, d.vo2_max,
                d.activities_count, d.activities_json,
                d.sleep_restless_moments, d.sleep_avg_overnight_hr, d.skin_temp_overnight,
                d.sleep_score_feedback, d.training_readiness_feedback,
                synced_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_daily(&self, user_id: Uuid, date: &str) -> anyhow::Result<Option<GarminDailyData>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM garmin_daily_data WHERE user_id = ?1 AND date = ?2"
        )?;
        let result = stmt.query_row(rusqlite::params![user_id.to_string(), date], |row| {
            row_to_daily_data(row)
        });
        match result {
            Ok(data) => Ok(Some(data)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_daily_range(&self, user_id: Uuid, start: &str, end: &str) -> anyhow::Result<Vec<GarminDailyData>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM garmin_daily_data WHERE user_id = ?1 AND date >= ?2 AND date <= ?3 ORDER BY date ASC"
        )?;
        let rows = stmt.query_map(rusqlite::params![user_id.to_string(), start, end], |row| {
            row_to_daily_data(row)
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_existing_dates(&self, user_id: Uuid, start: &str, end: &str) -> anyhow::Result<Vec<String>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT date FROM garmin_daily_data WHERE user_id = ?1 AND date >= ?2 AND date <= ?3"
        )?;
        let dates = stmt.query_map(rusqlite::params![user_id.to_string(), start, end], |row| {
            row.get(0)
        })?.collect::<Result<Vec<String>, _>>()?;
        Ok(dates)
    }

    pub fn get_recently_synced_dates(&self, user_id: Uuid, start: &str, end: &str, min_synced_at: f64) -> anyhow::Result<Vec<String>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT date FROM garmin_daily_data WHERE user_id = ?1 AND date >= ?2 AND date <= ?3 AND synced_at >= ?4"
        )?;
        let dates = stmt.query_map(rusqlite::params![user_id.to_string(), start, end, min_synced_at], |row| {
            row.get(0)
        })?.collect::<Result<Vec<String>, _>>()?;
        Ok(dates)
    }

    pub fn get_baseline(&self, user_id: Uuid, reference_date: &str, days: i64) -> anyhow::Result<Baseline> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT
                AVG(hrv_last_night),
                AVG(resting_heart_rate),
                AVG(avg_stress),
                AVG(body_battery_high),
                AVG(sleep_score),
                AVG(steps),
                AVG(sleep_duration_secs),
                COUNT(*)
             FROM garmin_daily_data
             WHERE user_id = ?1
             AND date >= date(?2, '-' || ?3 || ' days')
             AND date < ?2"
        )?;
        let baseline = stmt.query_row(
            rusqlite::params![user_id.to_string(), reference_date, days],
            |row| {
                Ok(Baseline {
                    avg_hrv: row.get(0)?,
                    avg_rhr: row.get(1)?,
                    avg_stress: row.get(2)?,
                    avg_body_battery: row.get(3)?,
                    avg_sleep_score: row.get(4)?,
                    avg_steps: row.get(5)?,
                    avg_sleep_hours: row.get::<_, Option<f64>>(6)?.map(|s| s / 3600.0),
                    days_counted: row.get(7)?,
                })
            },
        )?;
        Ok(baseline)
    }
}

fn row_to_daily_data(row: &rusqlite::Row) -> rusqlite::Result<GarminDailyData> {
    let user_id_str: String = row.get(0)?;
    let date_str: String = row.get(1)?;
    let synced_at: f64 = row.get(44)?;

    Ok(GarminDailyData {
        user_id: uuid::Uuid::parse_str(&user_id_str).unwrap_or_default(),
        date: chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Utc::now().date_naive()),
        steps: row.get(2)?,
        distance_meters: row.get(3)?,
        active_calories: row.get(4)?,
        total_calories: row.get(5)?,
        floors_climbed: row.get(6)?,
        intensity_minutes: row.get(7)?,
        resting_heart_rate: row.get(8)?,
        max_heart_rate: row.get(9)?,
        min_heart_rate: row.get(10)?,
        avg_heart_rate: row.get(11)?,
        hrv_weekly_avg: row.get(12)?,
        hrv_last_night: row.get(13)?,
        hrv_status: row.get(14)?,
        sleep_score: row.get(15)?,
        sleep_duration_secs: row.get(16)?,
        deep_sleep_secs: row.get(17)?,
        light_sleep_secs: row.get(18)?,
        rem_sleep_secs: row.get(19)?,
        awake_secs: row.get(20)?,
        avg_stress: row.get(21)?,
        max_stress: row.get(22)?,
        body_battery_high: row.get(23)?,
        body_battery_low: row.get(24)?,
        body_battery_drain: row.get(25)?,
        body_battery_charge: row.get(26)?,
        weight_grams: row.get(27)?,
        bmi: row.get(28)?,
        body_fat_pct: row.get(29)?,
        muscle_mass_grams: row.get(30)?,
        avg_spo2: row.get(31)?,
        lowest_spo2: row.get(32)?,
        avg_respiration: row.get(33)?,
        training_readiness: row.get(34)?,
        training_load: row.get(35)?,
        vo2_max: row.get(36)?,
        activities_count: row.get(37)?,
        activities_json: row.get(38)?,
        sleep_restless_moments: row.get(39)?,
        sleep_avg_overnight_hr: row.get(40)?,
        skin_temp_overnight: row.get(41)?,
        sleep_score_feedback: row.get(42)?,
        training_readiness_feedback: row.get(43)?,
        synced_at: chrono::DateTime::from_timestamp(synced_at as i64, 0)
            .unwrap_or_else(chrono::Utc::now),
    })
}
