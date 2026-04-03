use std::sync::Arc;
use crate::garmin::{self, GarminApiError};
use crate::events;
use crate::state::AppState;

/// Perform a full Garmin sync for a user (rate-limited, smart date skipping)
pub async fn perform_user_sync(state: &Arc<AppState>, user_id: uuid::Uuid) -> Result<String, anyhow::Error> {
    let rate_limit_mins = state.config.sync_rate_limit_mins;

    if let Ok(Some(last_sync)) = state.repo.get_last_sync(user_id) {
        let now = chrono::Utc::now().timestamp() as f64;
        let elapsed_mins = (now - last_sync) / 60.0;
        if elapsed_mins < rate_limit_mins as f64 {
            let mins_left = rate_limit_mins as f64 - elapsed_mins;
            return Ok(format!("Rate limited. Next sync in ~{:.0} min.", mins_left));
        }
    }

    tracing::info!("perform_user_sync: starting sync for user {}", user_id);

    let vault = &state.vault;

    let user = match state.repo.get_user(user_id)? {
        Some(u) => u,
        None => return Ok("User not found".into()),
    };

    let client = &state.http_client;

    // Try stored session first, fall back to re-login
    let mut stored_session: Option<garmin::GarminSession> = if let (Some(enc_tok), Some(tok_nonce)) =
        (&user.encrypted_session, &user.session_nonce)
    {
        match vault.decrypt(enc_tok, tok_nonce) {
            Ok(json_str) => {
                if let Ok(session) = serde_json::from_str::<garmin::GarminSession>(&json_str) {
                    Some(session)
                } else if let Ok(tok) = serde_json::from_str::<garmin::GarminOAuth2Token>(&json_str) {
                    tracing::info!("Migrating legacy GarminOAuth2Token to GarminSession");
                    Some(garmin::GarminSession {
                        oauth2: tok,
                        oauth1_token: String::new(),
                        oauth1_token_secret: String::new(),
                        oauth2_created_at: 0,
                    })
                } else {
                    tracing::warn!("Stored Garmin token is invalid JSON, will try re-login");
                    None
                }
            }
            Err(_) => return Err(anyhow::anyhow!("Could not decrypt OAuth token. Check MASTER_KEY.")),
        }
    } else {
        None
    };

    // If no stored session, try password login
    if stored_session.is_none() {
        let pass = vault.decrypt(&user.encrypted_password, &user.password_nonce)?;
        match garmin::garmin_login(&user.garmin_username, &pass).await {
            garmin::LoginResult::Success(session) => {
                let session_json = serde_json::to_string(&session).unwrap_or_default();
                let (enc, n) = vault.encrypt(&session_json)?;
                let _ = state.repo.save_session(user_id, &enc, &n);
                stored_session = Some(session);
            }
            garmin::LoginResult::MfaRequired { .. } => {
                let _ = state.repo.update_status(user_id, "mfa_required");
                return Err(anyhow::anyhow!("MFA required. Submit MFA code via API."));
            }
            garmin::LoginResult::Error(msg) => {
                let _ = state.repo.update_status(user_id, "expired");
                return Err(anyhow::anyhow!("Garmin login failed: {}", msg));
            }
        }
    }

    let mut session = match stored_session {
        Some(s) => s,
        None => return Err(anyhow::anyhow!("No Garmin session available after login attempt")),
    };

    // Proactive OAuth2 refresh
    if session.is_oauth2_expired() && session.has_oauth1_creds() {
        tracing::info!("Garmin OAuth2 token expired, refreshing via OAuth1 credentials");
        match garmin::refresh_oauth2_token(client, &session).await {
            Ok(new_session) => {
                let session_json = serde_json::to_string(&new_session).unwrap_or_default();
                let (enc, n) = vault.encrypt(&session_json)?;
                let _ = state.repo.save_session(user_id, &enc, &n);
                session = new_session;
            }
            Err(e) => {
                tracing::warn!("Proactive OAuth2 refresh failed: {} -- will try API call anyway", e);
            }
        }
    }

    let mut access_token = session.oauth2.access_token.clone();

    // Get display name, handle auth failures
    let display_name = match garmin::get_display_name(client, &access_token).await {
        Ok(name) => name,
        Err(GarminApiError::AuthFailed) => {
            tracing::info!("Garmin access token got 401, attempting refresh");
            if session.has_oauth1_creds() {
                match garmin::refresh_oauth2_token(client, &session).await {
                    Ok(new_session) => {
                        access_token = new_session.oauth2.access_token.clone();
                        let session_json = serde_json::to_string(&new_session).unwrap_or_default();
                        let (enc, n) = vault.encrypt(&session_json)?;
                        let _ = state.repo.save_session(user_id, &enc, &n);
                        session = new_session;
                        garmin::get_display_name(client, &access_token).await
                            .map_err(|e| anyhow::anyhow!("Garmin API failed after token refresh: {}", e))?
                    }
                    Err(refresh_err) => {
                        tracing::warn!("OAuth1 refresh failed: {}", refresh_err);
                        // Try password re-login
                        let pass = vault.decrypt(&user.encrypted_password, &user.password_nonce)?;
                        match garmin::garmin_login(&user.garmin_username, &pass).await {
                            garmin::LoginResult::Success(new_session) => {
                                access_token = new_session.oauth2.access_token.clone();
                                let session_json = serde_json::to_string(&new_session).unwrap_or_default();
                                let (enc, n) = vault.encrypt(&session_json)?;
                                let _ = state.repo.save_session(user_id, &enc, &n);
                                session = new_session;
                                garmin::get_display_name(client, &access_token).await
                                    .map_err(|e| anyhow::anyhow!("Garmin API failed after re-login: {}", e))?
                            }
                            garmin::LoginResult::MfaRequired { .. } => {
                                let _ = state.repo.update_status(user_id, "mfa_required");
                                return Err(anyhow::anyhow!("MFA required"));
                            }
                            garmin::LoginResult::Error(msg) => {
                                let _ = state.repo.update_status(user_id, "expired");
                                return Err(anyhow::anyhow!("Garmin re-login failed: {}", msg));
                            }
                        }
                    }
                }
            } else {
                let _ = state.repo.update_status(user_id, "expired");
                return Err(anyhow::anyhow!("OAuth2 expired, no OAuth1 credentials for refresh"));
            }
        }
        Err(GarminApiError::RateLimited) => {
            return Err(anyhow::anyhow!("Garmin API rate limited (429)"));
        }
        Err(e @ GarminApiError::NetworkError(_)) => {
            return Err(anyhow::anyhow!("Garmin API unreachable: {}", e));
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Garmin API error: {}", e));
        }
    };

    let _ = &session;

    let today = chrono::Utc::now().date_naive();
    let yesterday = today - chrono::Duration::days(1);
    let sync_days = state.config.sync_days;
    let start = today - chrono::Duration::days(sync_days - 1);
    let api_delay = std::time::Duration::from_secs(state.config.garmin_api_delay_secs);
    let mut synced_days = 0u64;
    let mut errors = Vec::new();

    let start_str = start.format("%Y-%m-%d").to_string();
    let today_str = today.format("%Y-%m-%d").to_string();
    let yesterday_str = yesterday.format("%Y-%m-%d").to_string();

    let existing_dates: std::collections::HashSet<String> = state.repo
        .get_existing_dates(user_id, &start_str, &today_str)
        .unwrap_or_default()
        .into_iter()
        .collect();

    let recent_threshold = (chrono::Utc::now() - chrono::Duration::minutes(rate_limit_mins)).timestamp() as f64;
    let recently_synced: std::collections::HashSet<String> = state.repo
        .get_recently_synced_dates(user_id, &start_str, &today_str, recent_threshold)
        .unwrap_or_default()
        .into_iter()
        .collect();

    let mut is_first_day = true;
    let mut consecutive_empty = 0u64;
    let max_consecutive_empty = state.config.max_consecutive_empty_days;

    let start_time = std::time::Instant::now();

    for offset in 0..sync_days {
        let date = start + chrono::Duration::days(offset);
        let date_str = date.format("%Y-%m-%d").to_string();

        if existing_dates.contains(&date_str) {
            if date_str != today_str && date_str != yesterday_str {
                continue;
            }
            if recently_synced.contains(&date_str) {
                continue;
            }
        }

        if !is_first_day && api_delay > std::time::Duration::ZERO {
            tokio::time::sleep(api_delay).await;
        }
        is_first_day = false;

        tracing::info!("perform_user_sync: fetching data for {}", date_str);
        let payload = garmin::fetch_all_daily_data(client, &access_token, &date_str, user_id, &display_name).await;

        if payload.rate_limited {
            tracing::error!("perform_user_sync: rate limited (429) while fetching {} -- stopping sync", date_str);
            errors.push(format!("{}: rate limited (429), stopped", date_str));
            break;
        }

        if !payload.daily.has_data() {
            consecutive_empty += 1;
            if consecutive_empty >= max_consecutive_empty {
                errors.push(format!("aborted after {} consecutive empty days", consecutive_empty));
                break;
            }
            continue;
        }

        consecutive_empty = 0;

        match state.repo.upsert_garmin_daily(&payload.daily) {
            Ok(_) => {
                synced_days += 1;

                // Store intraday data if enabled and within intraday range
                if state.config.sync_intraday {
                    let intraday_start = today - chrono::Duration::days(state.config.sync_intraday_days);
                    if date >= intraday_start {
                        let _ = state.repo.upsert_intraday_hr(user_id, &date_str, &payload.intraday_hr);
                        let _ = state.repo.upsert_intraday_stress(user_id, &date_str, &payload.intraday_stress);
                        let _ = state.repo.upsert_intraday_steps(user_id, &date_str, &payload.intraday_steps);
                        let _ = state.repo.upsert_intraday_respiration(user_id, &date_str, &payload.intraday_respiration);
                        let _ = state.repo.upsert_intraday_hrv(user_id, &date_str, &payload.intraday_hrv);
                        let _ = state.repo.upsert_intraday_sleep(user_id, &date_str, &payload.intraday_sleep);
                        tracing::debug!("Stored intraday: HR={} stress={} steps={} resp={} hrv={} sleep={}",
                            payload.intraday_hr.len(), payload.intraday_stress.len(),
                            payload.intraday_steps.len(), payload.intraday_respiration.len(),
                            payload.intraday_hrv.len(), payload.intraday_sleep.len());
                    }
                }

                // Store extended daily data
                let _ = state.repo.upsert_daily_extended(&payload.extended);

                // Fetch and store GPS tracks for activities with distance
                if state.config.sync_gps_tracks && !payload.rate_limited {
                    if let Some(ref acts_json) = payload.daily.activities_json {
                        if let Ok(acts) = serde_json::from_str::<Vec<serde_json::Value>>(acts_json) {
                            for act in &acts {
                                let activity_id = act["id"].as_i64().unwrap_or(0);
                                let distance = act["distance_m"].as_f64().unwrap_or(0.0);
                                if activity_id > 0 && distance > 0.0 {
                                    if api_delay > std::time::Duration::ZERO {
                                        tokio::time::sleep(api_delay).await;
                                    }
                                    match garmin::fetch_activity_gps_track(client, &access_token, activity_id).await {
                                        Ok(points) if !points.is_empty() => {
                                            let uid = user_id.to_string();
                                            match state.repo.upsert_gps_track(activity_id, &uid, &date_str, &points) {
                                                Ok(_) => tracing::debug!("Stored GPS track for activity {}: {} points", activity_id, points.len()),
                                                Err(e) => tracing::warn!("Failed to store GPS track for activity {}: {}", activity_id, e),
                                            }
                                        }
                                        Ok(_) => { tracing::debug!("No GPS points for activity {}", activity_id); }
                                        Err(garmin::GarminApiError::RateLimited) => {
                                            tracing::warn!("Rate limited while fetching GPS track for activity {}", activity_id);
                                            break;
                                        }
                                        Err(e) => { tracing::debug!("GPS track fetch failed for activity {}: {}", activity_id, e); }
                                    }
                                }
                            }
                        }
                    }
                }

                // Emit daily_data_synced event
                let event = events::Event::new(
                    "daily_data_synced",
                    user_id,
                    serde_json::to_value(&payload.daily).unwrap_or_default(),
                );
                state.webhook_dispatcher.dispatch(&state.repo, event).await;
            }
            Err(e) => errors.push(format!("{}: {}", date_str, e)),
        }
    }

    let now = chrono::Utc::now().timestamp() as f64;
    let _ = state.repo.set_last_sync(user_id, now);
    let _ = state.repo.update_status(user_id, "connected");

    let duration = start_time.elapsed().as_secs_f64();

    // Emit sync_completed event
    let event = events::Event::new(
        "sync_completed",
        user_id,
        serde_json::json!({
            "days_synced": synced_days,
            "errors": errors.len(),
            "duration_secs": duration,
        }),
    );
    state.webhook_dispatcher.dispatch(&state.repo, event).await;

    if errors.is_empty() {
        Ok(format!("Synced {} days from Garmin Connect.", synced_days))
    } else {
        Ok(format!("Synced {} days. {} errors: {}", synced_days, errors.len(), errors.join("; ")))
    }
}

/// Background sync loop: runs hourly for all registered users
pub async fn background_sync_loop(state: Arc<AppState>) {
    // Initial delay to let server fully start
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60 * 60));
    interval.tick().await; // first tick is immediate

    loop {
        interval.tick().await;
        tracing::info!("background_sync: starting sync cycle");

        match state.repo.get_all_users() {
            Ok(users) => {
                for user in users {
                    if user.status == "mfa_required" {
                        tracing::debug!("background_sync: skipping user {} (MFA required)", user.user_id);
                        continue;
                    }
                    if let Err(e) = perform_user_sync(&state, user.user_id).await {
                        tracing::warn!("background_sync: sync failed for user {}: {}", user.user_id, e);
                        // Emit sync_failed event
                        let event = events::Event::new(
                            "sync_failed",
                            user.user_id,
                            serde_json::json!({ "reason": e.to_string() }),
                        );
                        state.webhook_dispatcher.dispatch(&state.repo, event).await;
                    }
                }
            }
            Err(e) => tracing::error!("background_sync: failed to get users: {}", e),
        }
    }
}
