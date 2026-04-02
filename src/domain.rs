use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Comprehensive daily data from Garmin Connect
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GarminDailyData {
    pub user_id: Uuid,
    pub date: NaiveDate,
    // Steps & Activity
    pub steps: Option<i64>,
    pub distance_meters: Option<f64>,
    pub active_calories: Option<i64>,
    pub total_calories: Option<i64>,
    pub floors_climbed: Option<i64>,
    pub intensity_minutes: Option<i64>,
    // Heart Rate
    pub resting_heart_rate: Option<i64>,
    pub max_heart_rate: Option<i64>,
    pub min_heart_rate: Option<i64>,
    pub avg_heart_rate: Option<i64>,
    // HRV
    pub hrv_weekly_avg: Option<f64>,
    pub hrv_last_night: Option<f64>,
    pub hrv_status: Option<String>,
    // Sleep
    pub sleep_score: Option<i64>,
    pub sleep_duration_secs: Option<i64>,
    pub deep_sleep_secs: Option<i64>,
    pub light_sleep_secs: Option<i64>,
    pub rem_sleep_secs: Option<i64>,
    pub awake_secs: Option<i64>,
    // Stress & Body Battery
    pub avg_stress: Option<i64>,
    pub max_stress: Option<i64>,
    pub body_battery_high: Option<i64>,
    pub body_battery_low: Option<i64>,
    pub body_battery_drain: Option<i64>,
    pub body_battery_charge: Option<i64>,
    // Body Composition
    pub weight_grams: Option<f64>,
    pub bmi: Option<f64>,
    pub body_fat_pct: Option<f64>,
    pub muscle_mass_grams: Option<f64>,
    // Respiration & SpO2
    pub avg_spo2: Option<f64>,
    pub lowest_spo2: Option<f64>,
    pub avg_respiration: Option<f64>,
    // Training
    pub training_readiness: Option<f64>,
    pub training_load: Option<f64>,
    pub vo2_max: Option<f64>,
    // Activities
    pub activities_count: Option<i64>,
    pub activities_json: Option<String>,
    // Sleep details
    pub sleep_restless_moments: Option<i64>,
    pub sleep_avg_overnight_hr: Option<f64>,
    pub skin_temp_overnight: Option<f64>,
    // Sync metadata
    pub synced_at: DateTime<Utc>,
}

/// A single Garmin activity parsed from `activities_json`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GarminActivity {
    pub name: Option<String>,
    #[serde(alias = "type")]
    pub activity_type: Option<String>,
    pub total_time_secs: Option<f64>,
    pub duration_secs: Option<f64>,
    pub work_time_secs: Option<f64>,
    pub rest_time_secs: Option<f64>,
    pub avg_hr: Option<i64>,
    pub max_hr: Option<i64>,
    pub primary_benefit: Option<String>,
    pub training_effect_aerobic: Option<f64>,
    pub training_effect_anaerobic: Option<f64>,
    pub exercise_load: Option<f64>,
    pub distance_m: Option<f64>,
    pub resting_calories: Option<i64>,
    pub active_calories: Option<i64>,
    pub total_calories: Option<i64>,
    pub calories: Option<i64>,
    pub est_sweat_loss_ml: Option<f64>,
    pub total_sets: Option<i64>,
    pub total_reps: Option<i64>,
    pub total_volume_kg: Option<f64>,
    pub avg_time_per_set_secs: Option<f64>,
    pub exercises: Option<Vec<serde_json::Value>>,
    pub moderate_intensity_mins: Option<i64>,
    pub vigorous_intensity_mins: Option<i64>,
    pub total_intensity_mins: Option<i64>,
    pub body_battery_start: Option<i64>,
    pub body_battery_end: Option<i64>,
    pub hr_zones: Option<Vec<serde_json::Value>>,
}

impl GarminDailyData {
    /// Returns true if at least one data field (not user_id/date/synced_at) has a value
    pub fn has_data(&self) -> bool {
        self.steps.is_some() || self.distance_meters.is_some() || self.active_calories.is_some()
            || self.total_calories.is_some() || self.floors_climbed.is_some()
            || self.resting_heart_rate.is_some() || self.avg_heart_rate.is_some()
            || self.hrv_weekly_avg.is_some() || self.hrv_last_night.is_some()
            || self.sleep_score.is_some() || self.sleep_duration_secs.is_some()
            || self.avg_stress.is_some() || self.body_battery_high.is_some()
            || self.weight_grams.is_some() || self.avg_spo2.is_some()
            || self.training_readiness.is_some() || self.vo2_max.is_some()
            || self.activities_count.is_some()
    }
}

/// Vitals response for life_manager (today's data + 7-day baseline)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VitalsResponse {
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
    // 7-day baseline
    pub baseline_hrv: Option<f64>,
    pub baseline_rhr: Option<f64>,
    pub baseline_stress: Option<f64>,
    pub baseline_battery: Option<f64>,
    pub baseline_sleep: Option<f64>,
}

/// Baseline averages over N days
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Baseline {
    pub avg_hrv: Option<f64>,
    pub avg_rhr: Option<f64>,
    pub avg_stress: Option<f64>,
    pub avg_body_battery: Option<f64>,
    pub avg_sleep_score: Option<f64>,
    pub avg_steps: Option<f64>,
    pub avg_sleep_hours: Option<f64>,
    pub days_counted: i64,
}
