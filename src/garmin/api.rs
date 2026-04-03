use std::error::Error;
use reqwest::Client;
use super::{CONNECT_API, USER_AGENT, GarminApiError};

/// Parse a JSON value as i64, falling back to f64->i64 cast.
fn json_i64(v: &serde_json::Value) -> Option<i64> {
    v.as_i64().or_else(|| v.as_f64().map(|x| x as i64))
}

/// Classify a reqwest error as network (retryable) or other.
fn classify_reqwest_error(e: &reqwest::Error) -> GarminApiError {
    if e.is_connect() || e.is_timeout() {
        GarminApiError::NetworkError(format!("{}", e))
    } else if let Some(source) = e.source() {
        let msg = source.to_string();
        if msg.contains("dns") || msg.contains("resolve") || msg.contains("connect") {
            GarminApiError::NetworkError(format!("{}", e))
        } else {
            GarminApiError::Other(format!("API request failed: {}", e))
        }
    } else {
        GarminApiError::Other(format!("API request failed: {}", e))
    }
}

/// Fetch data from Garmin Connect API using an OAuth2 token, with optional query params.
/// Retries up to 2 times on network errors.
pub async fn garmin_api(
    client: &Client,
    access_token: &str,
    path: &str,
    params: &[(&str, &str)],
) -> Result<serde_json::Value, GarminApiError> {
    let url = format!("{}{}", CONNECT_API, path);

    let mut last_err = GarminApiError::Other("no attempts made".into());
    for attempt in 1..=3 {
        match client
            .get(&url)
            .bearer_auth(access_token)
            .header("User-Agent", USER_AGENT)
            .query(params)
            .send()
            .await
        {
            Ok(resp) => return parse_api_response(resp, path).await,
            Err(e) => {
                last_err = classify_reqwest_error(&e);
                if last_err.is_network_error() && attempt < 3 {
                    tracing::warn!(
                        path,
                        attempt,
                        error = %e,
                        "Garmin API network error, retrying in {}s",
                        attempt * 2
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(attempt * 2)).await;
                    continue;
                }
                return Err(last_err);
            }
        }
    }
    Err(last_err)
}

/// Parse the HTTP response from a Garmin API call.
async fn parse_api_response(
    resp: reqwest::Response,
    path: &str,
) -> Result<serde_json::Value, GarminApiError> {
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        tracing::warn!("garmin API {} returned {} -- body: {}", path, status, &body[..body.len().min(500)]);
        return Err(match status.as_u16() {
            429 => {
                tracing::error!("garmin API {} rate limited (429) -- stopping further requests", path);
                GarminApiError::RateLimited
            }
            401 => GarminApiError::AuthFailed,
            code if code >= 500 => GarminApiError::ServerError(code),
            _ => GarminApiError::Other(format!("API {} returned {}", path, status)),
        });
    }

    resp.json()
        .await
        .map_err(|e| GarminApiError::Other(format!("Failed to parse response for {}: {}", path, e)))
}

/// Convenience wrapper: fetch without query params.
pub async fn garmin_connect_api(
    client: &Client,
    access_token: &str,
    path: &str,
) -> Result<serde_json::Value, GarminApiError> {
    garmin_api(client, access_token, path, &[]).await
}

/// Convenience wrapper: fetch with query params.
pub async fn garmin_api_with_params(
    client: &Client,
    access_token: &str,
    path: &str,
    params: &[(&str, &str)],
) -> Result<serde_json::Value, GarminApiError> {
    garmin_api(client, access_token, path, params).await
}

/// Fetch the user's display name (needed for some endpoints).
pub async fn get_display_name(client: &Client, access_token: &str) -> Result<String, GarminApiError> {
    match garmin_connect_api(client, access_token, "/userprofile-service/socialProfile").await {
        Ok(profile) => {
            if let Some(name) = profile["displayName"].as_str() {
                return Ok(name.to_string());
            }
            if let Some(name) = profile["userName"].as_str() {
                return Ok(name.to_string());
            }
        }
        Err(GarminApiError::AuthFailed) => return Err(GarminApiError::AuthFailed),
        Err(GarminApiError::RateLimited) => return Err(GarminApiError::RateLimited),
        Err(e @ GarminApiError::NetworkError(_)) => return Err(e),
        Err(_) => {}
    }
    match garmin_connect_api(client, access_token, "/userprofile-service/userprofile/user-settings").await {
        Ok(profile) => {
            if let Some(name) = profile["userData"]["displayName"].as_str() {
                return Ok(name.to_string());
            }
            if let Some(id) = profile["id"].as_i64() {
                return Ok(id.to_string());
            }
            Err(GarminApiError::Other("Could not determine Garmin display name from any endpoint".to_string()))
        }
        Err(e) => Err(e),
    }
}

/// Fetch all available daily data for a given date. Returns a partially-filled GarminDailyData
/// and a bool indicating whether a 429 rate limit was hit.
pub async fn fetch_all_daily_data(
    client: &Client,
    access_token: &str,
    date: &str,
    user_id: uuid::Uuid,
    display_name: &str,
) -> crate::domain::DailySyncPayload {
    use crate::domain::*;
    use chrono::NaiveDate;

    let parsed_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Utc::now().date_naive());

    let mut data = GarminDailyData {
        user_id,
        date: parsed_date,
        steps: None, distance_meters: None, active_calories: None, total_calories: None,
        floors_climbed: None, intensity_minutes: None,
        resting_heart_rate: None, max_heart_rate: None, min_heart_rate: None, avg_heart_rate: None,
        hrv_weekly_avg: None, hrv_last_night: None, hrv_status: None,
        sleep_score: None, sleep_duration_secs: None, deep_sleep_secs: None,
        light_sleep_secs: None, rem_sleep_secs: None, awake_secs: None,
        avg_stress: None, max_stress: None,
        body_battery_high: None, body_battery_low: None, body_battery_drain: None, body_battery_charge: None,
        weight_grams: None, bmi: None, body_fat_pct: None, muscle_mass_grams: None,
        avg_spo2: None, lowest_spo2: None, avg_respiration: None,
        training_readiness: None, training_load: None, vo2_max: None,
        activities_count: None, activities_json: None,
        sleep_restless_moments: None, sleep_avg_overnight_hr: None, skin_temp_overnight: None,
        synced_at: chrono::Utc::now(),
    };

    let display_name = display_name.to_string();
    let mut rate_limited = false;

    // Intraday collections
    let mut intraday_hr: Vec<IntradayPoint> = Vec::new();
    let mut intraday_stress: Vec<StressPoint> = Vec::new();
    let mut intraday_steps: Vec<IntradayPoint> = Vec::new();
    let mut intraday_respiration: Vec<IntradayPointF64> = Vec::new();
    let mut intraday_hrv: Vec<HrvReading> = Vec::new();
    let mut intraday_sleep: Vec<SleepEpoch> = Vec::new();
    let mut extended = DailyExtended {
        user_id: user_id.to_string(),
        date: date.to_string(),
        ..Default::default()
    };

    fn was_rate_limited<T>(r: &Result<T, GarminApiError>) -> bool {
        matches!(r, Err(GarminApiError::RateLimited))
    }

    // Fire all 12 daily endpoints in parallel
    let summary_path = format!("/usersummary-service/usersummary/daily/{}", display_name);
    let hr_path = format!("/wellness-service/wellness/dailyHeartRate/{}", display_name);
    let hrv_path = format!("/hrv-service/hrv/{}", date);
    let sleep_path = format!("/wellness-service/wellness/dailySleepData/{}", display_name);
    let stress_path = format!("/wellness-service/wellness/dailyStress/{}", date);
    let bb_path = "/wellness-service/wellness/bodyBattery/reports/daily";
    let weight_path = "/weight-service/weight/dateRange";
    let spo2_path = format!("/wellness-service/wellness/daily/spo2/{}", date);
    let resp_path = format!("/wellness-service/wellness/daily/respiration/{}", date);
    let tr_path = format!("/metrics-service/metrics/trainingreadiness/{}", date);
    let ts_path = format!("/metrics-service/metrics/trainingstatus/aggregated/{}", date);
    let acts_path = "/activitylist-service/activities/search/activities";

    let summary_params: [(&str, &str); 1] = [("calendarDate", date)];
    let hr_params: [(&str, &str); 1] = [("date", date)];
    let sleep_params: [(&str, &str); 2] = [("date", date), ("nonSleepBufferMinutes", "60")];
    let bb_params: [(&str, &str); 2] = [("startDate", date), ("endDate", date)];
    let weight_params: [(&str, &str); 2] = [("startDate", date), ("endDate", date)];
    let acts_params: [(&str, &str); 4] = [("startDate", date), ("endDate", date), ("start", "0"), ("limit", "50")];

    let (
        summary_res, hr_res, hrv_res, sleep_res, stress_res, bb_res,
        weight_res, spo2_res, resp_res, tr_res, ts_res, acts_res,
    ) = tokio::join!(
        garmin_api_with_params(client, access_token, &summary_path, &summary_params),
        garmin_api_with_params(client, access_token, &hr_path, &hr_params),
        garmin_connect_api(client, access_token, &hrv_path),
        garmin_api_with_params(client, access_token, &sleep_path, &sleep_params),
        garmin_connect_api(client, access_token, &stress_path),
        garmin_api_with_params(client, access_token, bb_path, &bb_params),
        garmin_api_with_params(client, access_token, weight_path, &weight_params),
        garmin_connect_api(client, access_token, &spo2_path),
        garmin_connect_api(client, access_token, &resp_path),
        garmin_connect_api(client, access_token, &tr_path),
        garmin_connect_api(client, access_token, &ts_path),
        garmin_api_with_params(client, access_token, acts_path, &acts_params),
    );

    for res in [&summary_res, &hr_res, &hrv_res, &sleep_res, &stress_res, &bb_res,
                &weight_res, &spo2_res, &resp_res, &tr_res, &ts_res, &acts_res] {
        if was_rate_limited(res) { rate_limited = true; break; }
    }

    // 1. Daily Summary
    if let Ok(v) = summary_res {
        data.steps = json_i64(&v["totalSteps"]);
        data.distance_meters = v["totalDistanceMeters"].as_f64();
        data.active_calories = json_i64(&v["activeKilocalories"]);
        data.total_calories = json_i64(&v["totalKilocalories"]);
        data.floors_climbed = json_i64(&v["floorsAscended"]);
        let moderate = json_i64(&v["moderateIntensityMinutes"]);
        let vigorous = json_i64(&v["vigorousIntensityMinutes"]);
        data.intensity_minutes = moderate.zip(vigorous).map(|(m, v)| m + v * 2).or(moderate);
        data.body_battery_high = json_i64(&v["bodyBatteryHighestValue"]);
        data.body_battery_low = json_i64(&v["bodyBatteryLowestValue"]);
        data.body_battery_charge = json_i64(&v["bodyBatteryChargedValue"]);
        data.body_battery_drain = json_i64(&v["bodyBatteryDrainedValue"]);
        data.avg_spo2 = v["averageSpo2"].as_f64();
        data.lowest_spo2 = v["lowestSpo2"].as_f64();
        // Extended: activity durations from summary
        extended.sedentary_secs = json_i64(&v["sedentarySeconds"]);
        extended.active_secs = json_i64(&v["activeSeconds"]);
        extended.highly_active_secs = json_i64(&v["highlyActiveSeconds"]);
    }

    // 2. Heart Rate
    if let Ok(v) = hr_res {
        data.resting_heart_rate = json_i64(&v["restingHeartRate"]);
        data.max_heart_rate = json_i64(&v["maxHeartRate"]);
        data.min_heart_rate = json_i64(&v["minHeartRate"]);
        if let Some(values) = v["heartRateValues"].as_array() {
            let valid: Vec<i64> = values.iter()
                .filter_map(|pair| pair.as_array().and_then(|a| a.get(1)).and_then(|v| v.as_i64()))
                .filter(|&hr| hr > 0)
                .collect();
            if !valid.is_empty() {
                data.avg_heart_rate = Some(valid.iter().sum::<i64>() / valid.len() as i64);
            }
            // Intraday HR: collect all [timestamp_ms, hr] pairs
            for pair in values {
                if let Some(arr) = pair.as_array() {
                    if let (Some(ts), Some(hr)) = (arr.first().and_then(|v| v.as_i64()), arr.get(1).and_then(|v| v.as_i64())) {
                        if hr > 0 {
                            intraday_hr.push(IntradayPoint { ts, value: hr });
                        }
                    }
                }
            }
        }
    }

    // 3. HRV
    if let Ok(v) = hrv_res {
        data.hrv_weekly_avg = v["hrvSummary"]["weeklyAvg"].as_f64()
            .or_else(|| v["weeklyAvg"].as_f64());
        data.hrv_last_night = v["hrvSummary"]["lastNight"].as_f64()
            .or_else(|| v["lastNightAvg"].as_f64())
            .or_else(|| v["hrvSummary"]["lastNightAvg"].as_f64());
        data.hrv_status = v["hrvSummary"]["status"].as_str()
            .or_else(|| v["status"].as_str())
            .map(|s| s.to_string());
        // Intraday HRV readings
        if let Some(readings) = v["hrvReadings"].as_array()
            .or_else(|| v["hrvSummary"]["hrvReadings"].as_array()) {
            for r in readings {
                if let (Some(ts), Some(hrv)) = (
                    r["readingTimeGMT"].as_str().and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").ok()).map(|dt| dt.and_utc().timestamp_millis())
                        .or_else(|| r["startTimestampGMT"].as_i64()),
                    r["hrvValue"].as_f64().or_else(|| r["readingValue"].as_f64()),
                ) {
                    intraday_hrv.push(HrvReading { ts, hrv_value: hrv });
                }
            }
        }
    }

    // 4. Sleep
    if let Ok(v) = sleep_res {
        let dto = &v["dailySleepDTO"];
        data.sleep_score = v["sleepScores"]["overall"]["value"].as_i64()
            .or_else(|| v["sleepScores"]["overallScore"]["value"].as_i64())
            .or_else(|| dto["sleepScores"]["overall"]["value"].as_i64())
            .or_else(|| dto["sleepScores"]["overallScore"]["value"].as_i64())
            .or_else(|| v["sleepScoreDTO"]["overallScore"].as_i64());
        data.sleep_duration_secs = dto["sleepTimeSeconds"].as_i64()
            .or_else(|| v["sleepTimeSeconds"].as_i64());
        data.deep_sleep_secs = dto["deepSleepSeconds"].as_i64()
            .or_else(|| v["deepSleepSeconds"].as_i64());
        data.light_sleep_secs = dto["lightSleepSeconds"].as_i64()
            .or_else(|| v["lightSleepSeconds"].as_i64());
        data.rem_sleep_secs = dto["remSleepSeconds"].as_i64()
            .or_else(|| v["remSleepSeconds"].as_i64());
        data.awake_secs = dto["awakeSleepSeconds"].as_i64()
            .or_else(|| v["awakeSleepSeconds"].as_i64());
        data.sleep_restless_moments = dto["restlessMomentsCount"].as_i64()
            .or_else(|| v["restlessMomentsCount"].as_i64());
        data.sleep_avg_overnight_hr = dto["averageHeartRate"].as_f64()
            .or_else(|| dto["averageOvernightHR"].as_f64())
            .or_else(|| v["restingHeartRate"].as_f64());
        data.skin_temp_overnight = v["avgSkinTempDeviationC"].as_f64()
            .or_else(|| dto["avgSkinTempDeviationC"].as_f64())
            .or_else(|| v["skinTempDeviation"].as_f64())
            .or_else(|| dto["skinTempDeviation"].as_f64());
        if data.avg_spo2.is_none() {
            data.avg_spo2 = v["averageSpO2"].as_f64()
                .or_else(|| dto["averageSpO2Value"].as_f64());
        }
        if data.lowest_spo2.is_none() {
            data.lowest_spo2 = v["lowestSpO2"].as_f64()
                .or_else(|| dto["lowestSpO2Value"].as_f64());
        }
        if data.avg_respiration.is_none() {
            data.avg_respiration = dto["averageRespirationValue"].as_f64()
                .or_else(|| v["averageRespirationValue"].as_f64());
        }
        // Intraday sleep: stages, HR, SpO2, movement
        if let Some(levels) = v["sleepLevels"].as_array()
            .or_else(|| dto["sleepLevels"].as_array()) {
            for level in levels {
                let stage_val = level["activityLevel"].as_f64().or_else(|| level["activityLevel"].as_i64().map(|i| i as f64));
                let stage = stage_val.map(|v| match v as i64 {
                    0 => "deep", 1 => "light", 2 => "rem", _ => "awake",
                }.to_string());
                if let Some(ts) = level["startGMT"].as_str()
                    .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").ok())
                    .map(|dt| dt.and_utc().timestamp_millis())
                    .or_else(|| level["startGMT"].as_i64())
                {
                    intraday_sleep.push(SleepEpoch {
                        ts, stage,
                        hr: level["heartRate"].as_i64().or_else(|| json_i64(&level["heartRate"])),
                        spo2: level["spo2Reading"].as_f64(),
                        respiration: level["respirationValue"].as_f64(),
                        movement: level["activityLevel"].as_f64(),
                    });
                }
            }
        }
    }

    // 5. Stress
    if let Ok(v) = stress_res {
        data.avg_stress = json_i64(&v["overallStressLevel"])
            .or_else(|| json_i64(&v["avgStressLevel"]));
        data.max_stress = json_i64(&v["maxStressLevel"]);
        // Intraday stress values
        if let Some(arr) = v["stressValuesArray"].as_array() {
            for pair in arr {
                if let Some(a) = pair.as_array() {
                    if let (Some(ts), Some(stress)) = (a.first().and_then(|v| v.as_i64()), a.get(1).and_then(|v| v.as_i64())) {
                        if stress >= 0 {
                            intraday_stress.push(StressPoint { ts, stress, body_battery: None });
                        }
                    }
                }
            }
        }
        // Intraday body battery values (merge into stress points by timestamp)
        if let Some(arr) = v["bodyBatteryValuesArray"].as_array() {
            let mut bb_map: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
            for pair in arr {
                if let Some(a) = pair.as_array() {
                    if let (Some(ts), Some(bb)) = (a.first().and_then(|v| v.as_i64()), a.get(1).and_then(|v| v.as_i64())) {
                        bb_map.insert(ts, bb);
                    }
                }
            }
            for p in &mut intraday_stress {
                if let Some(&bb) = bb_map.get(&p.ts) {
                    p.body_battery = Some(bb);
                }
            }
            // Add body battery points that don't have matching stress
            for (ts, bb) in &bb_map {
                if !intraday_stress.iter().any(|p| p.ts == *ts) {
                    intraday_stress.push(StressPoint { ts: *ts, stress: -1, body_battery: Some(*bb) });
                }
            }
            intraday_stress.sort_by_key(|p| p.ts);
        }
        // Extended: stress duration breakdown
        extended.low_stress_secs = json_i64(&v["lowStressDuration"]);
        extended.medium_stress_secs = json_i64(&v["mediumStressDuration"]);
        extended.high_stress_secs = json_i64(&v["highStressDuration"]);
        extended.rest_stress_secs = json_i64(&v["restStressDuration"]);
    }

    // 6. Body Battery
    if let Ok(v) = bb_res
        && let Some(entries) = v.as_array()
            && let Some(entry) = entries.first() {
                data.body_battery_charge = json_i64(&entry["charged"])
                    .or_else(|| json_i64(&entry["bodyBatteryChargedValue"]));
                data.body_battery_drain = json_i64(&entry["drained"])
                    .or_else(|| json_i64(&entry["bodyBatteryDrainedValue"]));
                if let Some(high) = json_i64(&entry["bodyBatteryHighValue"])
                    .or_else(|| json_i64(&entry["highValue"])) {
                    data.body_battery_high = Some(high);
                }
                if let Some(low) = json_i64(&entry["bodyBatteryLowValue"])
                    .or_else(|| json_i64(&entry["lowValue"])) {
                    data.body_battery_low = Some(low);
                }
            }

    // 7. Body Composition / Weight
    if let Ok(v) = weight_res {
        let mut found = false;
        if let Some(entries) = v["dateWeightList"].as_array()
            .or_else(|| v["dailyWeightSummaries"].as_array())
            && let Some(entry) = entries.last() {
                let lw = &entry["latestWeight"];
                data.weight_grams = lw["weight"].as_f64()
                    .or_else(|| entry["weight"].as_f64());
                data.bmi = lw["bmi"].as_f64()
                    .or_else(|| entry["bmi"].as_f64());
                data.body_fat_pct = lw["bodyFat"].as_f64()
                    .or_else(|| entry["bodyFat"].as_f64());
                data.muscle_mass_grams = lw["muscleMass"].as_f64()
                    .or_else(|| entry["muscleMass"].as_f64());
                found = data.weight_grams.is_some();
            }
        if !found
            && let Some(entries) = v.as_array()
                && let Some(entry) = entries.last() {
                    data.weight_grams = entry["weight"].as_f64()
                        .or_else(|| entry["latestWeight"]["weight"].as_f64());
                    data.bmi = entry["bmi"].as_f64()
                        .or_else(|| entry["latestWeight"]["bmi"].as_f64());
                    data.body_fat_pct = entry["bodyFat"].as_f64()
                        .or_else(|| entry["latestWeight"]["bodyFat"].as_f64());
                    data.muscle_mass_grams = entry["muscleMass"].as_f64()
                        .or_else(|| entry["latestWeight"]["muscleMass"].as_f64());
                }
    }

    // 8. SpO2
    if let Ok(v) = spo2_res {
        data.avg_spo2 = v["averageSPO2"].as_f64()
            .or_else(|| v["averageSpO2"].as_f64());
        data.lowest_spo2 = v["lowestSPO2"].as_f64()
            .or_else(|| v["lowestSpO2"].as_f64());
    }

    // 9. Respiration
    if let Ok(v) = resp_res {
        data.avg_respiration = v["avgWakingRespirationValue"].as_f64()
            .or_else(|| v["avgSleepRespirationValue"].as_f64());
        // Intraday respiration
        if let Some(arr) = v["respirationValuesArray"].as_array() {
            for pair in arr {
                if let Some(a) = pair.as_array() {
                    if let (Some(ts), Some(val)) = (a.first().and_then(|v| v.as_i64()), a.get(1).and_then(|v| v.as_f64())) {
                        if val > 0.0 {
                            intraday_respiration.push(IntradayPointF64 { ts, value: val });
                        }
                    }
                }
            }
        }
    }

    // 10. Training Readiness
    if let Ok(v) = tr_res {
        let entry = if v.is_array() { v.as_array().and_then(|a| a.first().cloned()) } else { Some(v.clone()) };
        if let Some(e) = entry {
            data.training_readiness = e["score"].as_f64()
                .or_else(|| e["trainingReadinessScore"].as_f64());
        }
    }

    // 11. Training Status (VO2 max, load)
    if let Ok(v) = ts_res {
        let vo2_obj = &v["mostRecentVO2Max"];
        data.vo2_max = vo2_obj["generic"]["vo2MaxPreciseValue"].as_f64()
            .or_else(|| vo2_obj["generic"]["vo2MaxValue"].as_f64())
            .or_else(|| vo2_obj["cycling"]["vo2MaxPreciseValue"].as_f64())
            .or_else(|| vo2_obj["cycling"]["vo2MaxValue"].as_f64())
            .or_else(|| vo2_obj["vo2MaxPreciseValue"].as_f64())
            .or_else(|| vo2_obj.as_f64());

        if let Some(map) = v["mostRecentTrainingLoadBalance"]["metricsTrainingLoadBalanceDTOMap"].as_object()
            && let Some(entry) = map.values().next() {
                let aero_low = entry["monthlyLoadAerobicLow"].as_f64().unwrap_or(0.0);
                let aero_high = entry["monthlyLoadAerobicHigh"].as_f64().unwrap_or(0.0);
                let anaero = entry["monthlyLoadAnaerobic"].as_f64().unwrap_or(0.0);
                let total = aero_low + aero_high + anaero;
                if total > 0.0 {
                    data.training_load = Some(total);
                }
            }
        if data.training_load.is_none() {
            data.training_load = v["trainingLoad7Day"].as_f64()
                .or_else(|| v["acuteTrainingLoad"].as_f64());
        }
    }

    // 12. Activities
    if let Ok(v) = acts_res
        && let Some(arr) = v.as_array() {
            data.activities_count = Some(arr.len() as i64);
            let mut summaries: Vec<serde_json::Value> = Vec::new();
            for a in arr {
                let type_key = a["activityType"]["typeKey"].as_str().unwrap_or("");
                let activity_id = a["activityId"].as_i64().unwrap_or(0);
                let mut summary = serde_json::json!({
                    "id": activity_id,
                    "name": a["activityName"].as_str().unwrap_or(""),
                    "type": type_key,
                    "activityType": a["activityType"]["typeKey"].as_str().unwrap_or(""),
                    "duration_secs": a["duration"].as_f64().unwrap_or(0.0),
                    "distance_m": a["distance"].as_f64().unwrap_or(0.0),
                    "calories": json_i64(&a["calories"]).unwrap_or(0),
                    "avg_hr": json_i64(&a["averageHR"]).unwrap_or(0),
                    "max_hr": json_i64(&a["maxHR"]).unwrap_or(0),
                    "training_effect_aerobic": a["aerobicTrainingEffect"].as_f64().unwrap_or(0.0),
                    "training_effect_anaerobic": a["anaerobicTrainingEffect"].as_f64().unwrap_or(0.0),
                });

                // HR zone times from activity list
                {
                    let mut zone_secs: Vec<(i64, i64)> = Vec::new();
                    for i in 1..=5i64 {
                        let key = format!("hrTimeInZone_{}", i);
                        if let Some(secs) = json_i64(&a[&key])
                            && secs > 0 {
                                zone_secs.push((i, secs));
                            }
                    }
                    if !zone_secs.is_empty() {
                        let total: i64 = zone_secs.iter().map(|(_, s)| s).sum();
                        let zone_data: Vec<serde_json::Value> = zone_secs.iter().map(|(z, secs)| {
                            let pct = if total > 0 { (*secs as f64 / total as f64 * 1000.0).round() / 10.0 } else { 0.0 };
                            serde_json::json!({ "zone": z, "secs": secs, "pct": pct })
                        }).collect();
                        summary["hr_zones"] = serde_json::json!(zone_data);
                    }
                }

                // Enrichment sub-calls
                if activity_id > 0 && !rate_limited {
                    let detail_path = format!("/activity-service/activity/{}", activity_id);
                    match garmin_connect_api(client, access_token, &detail_path).await {
                        Ok(detail) => {
                            let s = &detail["summaryDTO"];
                            let is_strength = type_key.contains("strength") || type_key.contains("gym") || type_key.contains("weight");

                            let elapsed = s["elapsedDuration"].as_f64()
                                .or_else(|| s["duration"].as_f64())
                                .or_else(|| detail["elapsedDuration"].as_f64())
                                .unwrap_or(0.0);
                            let moving = s["movingDuration"].as_f64()
                                .or_else(|| detail["movingDuration"].as_f64())
                                .unwrap_or(elapsed);
                            summary["work_time_secs"] = serde_json::json!(moving);
                            summary["rest_time_secs"] = serde_json::json!(if elapsed > moving { elapsed - moving } else { 0.0 });
                            summary["total_time_secs"] = serde_json::json!(elapsed);
                            summary["exercise_load"] = serde_json::json!(detail["activityTrainingLoad"].as_f64()
                                .or_else(|| s["trainingEffect"].as_f64()));
                            summary["primary_benefit"] = serde_json::json!(detail["primaryBenefitDescription"].as_str()
                                .or_else(|| detail["primaryTrainingBenefit"].as_str()));
                            summary["resting_calories"] = serde_json::json!(json_i64(&s["bmrCalories"]));
                            summary["active_calories"] = serde_json::json!(json_i64(&s["activeKilocalories"])
                                .or_else(|| s["calories"].as_i64()));
                            summary["total_calories"] = serde_json::json!(json_i64(&s["calories"]));
                            summary["est_sweat_loss_ml"] = serde_json::json!(s["estimatedSweatLoss"].as_f64()
                                .or_else(|| detail["estimatedSweatLoss"].as_f64())
                                .or_else(|| s["estimatedSweatLossInMl"].as_f64())
                                .or_else(|| detail["estimatedSweatLossInMl"].as_f64()));
                            summary["moderate_intensity_mins"] = serde_json::json!(json_i64(&s["moderateIntensityMinutes"]));
                            summary["vigorous_intensity_mins"] = serde_json::json!(json_i64(&s["vigorousIntensityMinutes"]));
                            let mod_mins = s["moderateIntensityMinutes"].as_i64().unwrap_or(0);
                            let vig_mins = s["vigorousIntensityMinutes"].as_i64().unwrap_or(0);
                            if mod_mins > 0 || vig_mins > 0 {
                                summary["total_intensity_mins"] = serde_json::json!(mod_mins + vig_mins * 2);
                            }
                            summary["body_battery_start"] = serde_json::json!(detail["beginBodyBattery"].as_i64()
                                .or_else(|| s["startingBodyBattery"].as_i64())
                                .or_else(|| detail["startBodyBattery"].as_i64())
                                .or_else(|| s["beginBodyBattery"].as_i64()));
                            summary["body_battery_end"] = serde_json::json!(detail["endBodyBattery"].as_i64()
                                .or_else(|| s["endingBodyBattery"].as_i64())
                                .or_else(|| detail["endingBodyBattery"].as_i64())
                                .or_else(|| s["endBodyBattery"].as_i64()));

                            if is_strength {
                                let exercise_sets = detail["summarizedExerciseSets"].as_array()
                                    .or_else(|| detail["exerciseSets"].as_array())
                                    .or_else(|| detail["activityDetailGroups"].as_array());
                                if let Some(all_sets) = exercise_sets {
                                    parse_strength_sets(&mut summary, all_sets, activity_id, elapsed);
                                }
                            }

                            let hr_zones = detail["heartRateZones"].as_array()
                                .or_else(|| detail["heartRateZoneSummaries"].as_array())
                                .or_else(|| s["heartRateZones"].as_array());
                            if let Some(zones) = hr_zones {
                                parse_hr_zones(&mut summary, zones);
                            }

                            // Sub-endpoint fallbacks
                            if is_strength && !rate_limited && summary.get("total_sets").and_then(|v| v.as_u64()).is_none() {
                                let sets_path = format!("/activity-service/activity/{}/exerciseSets", activity_id);
                                match garmin_connect_api(client, access_token, &sets_path).await {
                                    Ok(sets_resp) => {
                                        let sets_arr = sets_resp["exerciseSets"].as_array()
                                            .or_else(|| sets_resp.as_array());
                                        if let Some(all_sets) = sets_arr
                                            && !all_sets.is_empty() {
                                                parse_strength_sets(&mut summary, all_sets, activity_id, elapsed);
                                            }
                                    }
                                    Err(GarminApiError::RateLimited) => { rate_limited = true; }
                                    Err(_) => {}
                                }
                            }

                            if summary.get("hr_zones").is_none() && !rate_limited && !is_strength {
                                let hrz_path = format!("/activity-service/activity/{}/heartRateZones", activity_id);
                                match garmin_connect_api(client, access_token, &hrz_path).await {
                                    Ok(hrz_resp) => {
                                        let zones = hrz_resp["heartRateZones"].as_array()
                                            .or_else(|| hrz_resp.as_array());
                                        if let Some(zones) = zones {
                                            parse_hr_zones(&mut summary, zones);
                                        }
                                    }
                                    Err(GarminApiError::RateLimited) => { rate_limited = true; }
                                    Err(_) => {}
                                }
                            }
                        }
                        Err(GarminApiError::RateLimited) => { rate_limited = true; }
                        Err(_) => {}
                    }
                }

                summaries.push(summary);
            }
            data.activities_json = Some(serde_json::to_string(&summaries).unwrap_or_default());
        }

    DailySyncPayload {
        daily: data,
        extended,
        intraday_hr,
        intraday_stress,
        intraday_steps,
        intraday_respiration,
        intraday_hrv,
        intraday_sleep,
        rate_limited,
    }
}

/// Parse work sets from exercise set data and populate summary fields
fn parse_strength_sets(summary: &mut serde_json::Value, all_sets: &[serde_json::Value], activity_id: i64, elapsed: f64) {
    let work_sets: Vec<&serde_json::Value> = all_sets.iter().filter(|s| {
        let is_rest = s["setType"].as_i64() == Some(2)
            || s["setType"].as_str().map(|t| t.eq_ignore_ascii_case("rest")).unwrap_or(false);
        if is_rest { return false; }
        let reps = s["reps"].as_i64()
            .or_else(|| s["repetitionCount"].as_i64())
            .or_else(|| s["numReps"].as_i64())
            .unwrap_or(0);
        reps > 0
    }).collect();
    tracing::debug!("Activity {} sets: {} total, {} work", activity_id, all_sets.len(), work_sets.len());

    summary["total_sets"] = serde_json::json!(work_sets.len());
    let total_reps: i64 = work_sets.iter().filter_map(|s|
        s["reps"].as_i64()
        .or_else(|| s["repetitionCount"].as_i64())
        .or_else(|| s["numReps"].as_i64())
    ).sum();
    if total_reps > 0 {
        summary["total_reps"] = serde_json::json!(total_reps);
    }
    let volume: f64 = work_sets.iter().map(|set| {
        let reps = set["reps"].as_f64()
            .or_else(|| set["repetitionCount"].as_f64())
            .or_else(|| set["numReps"].as_f64())
            .unwrap_or(0.0);
        let weight = set["weight"].as_f64()
            .or_else(|| set["weightUsed"].as_f64())
            .or_else(|| set["weightInGrams"].as_f64())
            .unwrap_or(0.0);
        reps * weight / 1000.0
    }).sum();
    if volume > 0.0 {
        summary["total_volume_kg"] = serde_json::json!(volume);
    }
    let num_sets = work_sets.len();
    if num_sets > 0 && elapsed > 0.0 {
        summary["avg_time_per_set_secs"] = serde_json::json!(elapsed / num_sets as f64);
    }

    let mut exercises: std::collections::BTreeMap<String, Vec<serde_json::Value>> = std::collections::BTreeMap::new();
    for set in &work_sets {
        let name = set["exerciseName"].as_str()
            .or_else(|| set["category"].as_str())
            .or_else(|| set["exerciseCategory"].as_str())
            .unwrap_or("Unknown")
            .to_string();
        let reps = set["reps"].as_i64()
            .or_else(|| set["repetitionCount"].as_i64())
            .or_else(|| set["numReps"].as_i64())
            .unwrap_or(0);
        let weight_g = set["weight"].as_f64()
            .or_else(|| set["weightUsed"].as_f64())
            .or_else(|| set["weightInGrams"].as_f64())
            .unwrap_or(0.0);
        let weight_kg = weight_g / 1000.0;
        exercises.entry(name).or_default().push(serde_json::json!({
            "reps": reps,
            "weight_kg": (weight_kg * 10.0).round() / 10.0,
        }));
    }
    let exercise_list: Vec<serde_json::Value> = exercises.into_iter().map(|(name, sets)| {
        serde_json::json!({ "exercise": name, "sets": sets })
    }).collect();
    if !exercise_list.is_empty() {
        summary["exercises"] = serde_json::json!(exercise_list);
    }
}

/// Parse HR zone data and populate summary
fn parse_hr_zones(summary: &mut serde_json::Value, zones: &[serde_json::Value]) {
    let total_secs: i64 = zones.iter().map(|z| z["secsInZone"].as_i64().unwrap_or(0)).sum();
    if total_secs > 0 {
        let zone_data: Vec<serde_json::Value> = zones.iter().map(|z| {
            let secs = z["secsInZone"].as_i64().unwrap_or(0);
            let pct = secs as f64 / total_secs as f64 * 100.0;
            serde_json::json!({
                "zone": z["zoneNumber"].as_i64().unwrap_or(0),
                "lo": z["zoneLowBoundary"].as_i64().unwrap_or(0),
                "hi": z["zoneHighBoundary"].as_i64().unwrap_or(0),
                "secs": secs,
                "pct": (pct * 10.0).round() / 10.0,
            })
        }).collect();
        summary["hr_zones"] = serde_json::json!(zone_data);
    }
}
