use serde::{Deserialize, Serialize};

// === Daily data (existing) ===

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VitalsData {
    pub hrv_last_night: Option<f64>,
    pub hrv_status: Option<String>,
    pub resting_heart_rate: Option<i64>,
    pub avg_stress: Option<i64>,
    pub body_battery_high: Option<i64>,
    pub body_battery_low: Option<i64>,
    pub sleep_score: Option<i64>,
    pub sleep_hours: Option<f64>,
    pub training_readiness: Option<f64>,
    pub steps: Option<i64>,
    pub baseline_hrv: Option<f64>,
    pub baseline_rhr: Option<f64>,
    pub baseline_stress: Option<f64>,
    pub baseline_battery: Option<f64>,
    pub baseline_sleep: Option<f64>,
    pub sleep_debt_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyData {
    pub date: String,
    pub steps: Option<i64>,
    pub distance_meters: Option<f64>,
    pub active_calories: Option<i64>,
    pub total_calories: Option<i64>,
    pub floors_climbed: Option<i64>,
    pub intensity_minutes: Option<i64>,
    pub resting_heart_rate: Option<i64>,
    pub max_heart_rate: Option<i64>,
    pub min_heart_rate: Option<i64>,
    pub avg_heart_rate: Option<i64>,
    pub hrv_last_night: Option<f64>,
    pub hrv_weekly_avg: Option<f64>,
    pub hrv_status: Option<String>,
    pub sleep_score: Option<i64>,
    pub sleep_duration_secs: Option<i64>,
    pub deep_sleep_secs: Option<i64>,
    pub light_sleep_secs: Option<i64>,
    pub rem_sleep_secs: Option<i64>,
    pub awake_secs: Option<i64>,
    pub avg_stress: Option<i64>,
    pub max_stress: Option<i64>,
    pub body_battery_high: Option<i64>,
    pub body_battery_low: Option<i64>,
    pub body_battery_charge: Option<i64>,
    pub body_battery_drain: Option<i64>,
    pub weight_grams: Option<f64>,
    pub bmi: Option<f64>,
    pub body_fat_pct: Option<f64>,
    pub muscle_mass_grams: Option<f64>,
    pub avg_spo2: Option<f64>,
    pub lowest_spo2: Option<f64>,
    pub avg_respiration: Option<f64>,
    pub training_readiness: Option<f64>,
    pub training_load: Option<f64>,
    pub vo2_max: Option<f64>,
    pub activities_count: Option<i64>,
    pub activities_json: Option<String>,
    pub sleep_restless_moments: Option<i64>,
    pub sleep_avg_overnight_hr: Option<f64>,
    pub sleep_score_feedback: Option<String>,
    pub training_readiness_feedback: Option<String>,
}

// === Intraday data ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntradayPoint {
    pub ts: i64,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntradayPointF64 {
    pub ts: f64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressPoint {
    pub ts: i64,
    pub stress: i64,
    pub body_battery: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HrvReading {
    pub ts: i64,
    pub hrv_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepEpoch {
    pub ts: i64,
    pub stage: Option<String>,
    pub hr: Option<i64>,
    pub spo2: Option<f64>,
    pub respiration: Option<f64>,
    pub movement: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntradayResponse<T> {
    pub date: String,
    pub points: Vec<T>,
}

// === Extended daily ===

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyExtended {
    pub user_id: Option<String>,
    pub date: Option<String>,
    pub fitness_age: Option<i64>,
    pub race_5k_secs: Option<f64>,
    pub race_10k_secs: Option<f64>,
    pub race_half_secs: Option<f64>,
    pub race_marathon_secs: Option<f64>,
    pub low_stress_secs: Option<i64>,
    pub medium_stress_secs: Option<i64>,
    pub high_stress_secs: Option<i64>,
    pub rest_stress_secs: Option<i64>,
    pub sedentary_secs: Option<i64>,
    pub active_secs: Option<i64>,
    pub highly_active_secs: Option<i64>,
}

// === Activity ===

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Activity {
    pub id: Option<i64>,
    pub date: Option<String>,
    pub name: Option<String>,
    #[serde(alias = "type")]
    pub activity_type: Option<String>,
    pub duration_secs: Option<f64>,
    pub distance_m: Option<f64>,
    pub calories: Option<i64>,
    pub avg_hr: Option<i64>,
    pub max_hr: Option<i64>,
    pub training_effect_aerobic: Option<f64>,
    pub training_effect_anaerobic: Option<f64>,
    pub total_sets: Option<i64>,
    pub total_reps: Option<i64>,
    pub total_volume_kg: Option<f64>,
    pub hr_zones: Option<Vec<serde_json::Value>>,
    pub exercises: Option<Vec<serde_json::Value>>,
    pub body_battery_start: Option<i64>,
    pub body_battery_end: Option<i64>,
}

// === User list ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarminUser {
    pub user_id: String,
    pub garmin_username: String,
    pub status: String,
    pub last_sync_at: Option<f64>,
}
