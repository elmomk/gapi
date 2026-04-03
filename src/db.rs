use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn init_pool(db_path: &str) -> DbPool {
    let manager = SqliteConnectionManager::file(db_path)
        .with_init(|conn| {
            conn.pragma_update(None, "busy_timeout", 5000)?;
            Ok(())
        });
    let pool = Pool::new(manager).expect("Failed to create DB pool");

    let conn = pool.get().expect("Failed to get DB connection");

    // WAL mode: crash-safe, better concurrent read/write performance
    conn.pragma_update(None, "journal_mode", "WAL")
        .expect("Failed to set WAL mode");
    conn.pragma_update(None, "synchronous", "NORMAL")
        .expect("Failed to set synchronous mode");
    conn.pragma_update(None, "busy_timeout", 5000)
        .expect("Failed to set busy_timeout");
    conn.pragma_update(None, "foreign_keys", "ON")
        .expect("Failed to enable foreign keys");

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS garmin_users (
            user_id TEXT PRIMARY KEY,
            garmin_username TEXT NOT NULL,
            encrypted_password TEXT NOT NULL,
            password_nonce TEXT NOT NULL,
            encrypted_session TEXT,
            session_nonce TEXT,
            status TEXT NOT NULL DEFAULT 'connected',
            last_sync_at REAL,
            created_at REAL NOT NULL,
            updated_at REAL NOT NULL
        );

        CREATE TABLE IF NOT EXISTS garmin_daily_data (
            user_id TEXT NOT NULL REFERENCES garmin_users(user_id) ON DELETE CASCADE,
            date TEXT NOT NULL,
            steps INTEGER,
            distance_meters REAL,
            active_calories INTEGER,
            total_calories INTEGER,
            floors_climbed INTEGER,
            intensity_minutes INTEGER,
            resting_heart_rate INTEGER,
            max_heart_rate INTEGER,
            min_heart_rate INTEGER,
            avg_heart_rate INTEGER,
            hrv_weekly_avg REAL,
            hrv_last_night REAL,
            hrv_status TEXT,
            sleep_score INTEGER,
            sleep_duration_secs INTEGER,
            deep_sleep_secs INTEGER,
            light_sleep_secs INTEGER,
            rem_sleep_secs INTEGER,
            awake_secs INTEGER,
            avg_stress INTEGER,
            max_stress INTEGER,
            body_battery_high INTEGER,
            body_battery_low INTEGER,
            body_battery_drain INTEGER,
            body_battery_charge INTEGER,
            weight_grams REAL,
            bmi REAL,
            body_fat_pct REAL,
            muscle_mass_grams REAL,
            avg_spo2 REAL,
            lowest_spo2 REAL,
            avg_respiration REAL,
            training_readiness REAL,
            training_load REAL,
            vo2_max REAL,
            activities_count INTEGER,
            activities_json TEXT,
            sleep_restless_moments INTEGER,
            sleep_avg_overnight_hr REAL,
            skin_temp_overnight REAL,
            synced_at REAL NOT NULL,
            PRIMARY KEY (user_id, date)
        );

        CREATE TABLE IF NOT EXISTS webhook_subscriptions (
            id TEXT PRIMARY KEY,
            consumer_name TEXT NOT NULL,
            url TEXT NOT NULL,
            secret TEXT,
            event_types TEXT NOT NULL,
            active INTEGER NOT NULL DEFAULT 1,
            created_at REAL NOT NULL
        );

        CREATE TABLE IF NOT EXISTS api_keys (
            key_hash TEXT PRIMARY KEY,
            consumer_name TEXT NOT NULL,
            created_at REAL NOT NULL
        );

        -- Intraday tables (per-minute granularity)
        CREATE TABLE IF NOT EXISTS intraday_heart_rate (
            user_id TEXT NOT NULL, date TEXT NOT NULL,
            ts_ms INTEGER NOT NULL, value INTEGER NOT NULL,
            PRIMARY KEY (user_id, date, ts_ms)
        ) WITHOUT ROWID;

        CREATE TABLE IF NOT EXISTS intraday_stress (
            user_id TEXT NOT NULL, date TEXT NOT NULL,
            ts_ms INTEGER NOT NULL, stress INTEGER NOT NULL, body_battery INTEGER,
            PRIMARY KEY (user_id, date, ts_ms)
        ) WITHOUT ROWID;

        CREATE TABLE IF NOT EXISTS intraday_steps (
            user_id TEXT NOT NULL, date TEXT NOT NULL,
            ts_ms INTEGER NOT NULL, steps INTEGER NOT NULL,
            PRIMARY KEY (user_id, date, ts_ms)
        ) WITHOUT ROWID;

        CREATE TABLE IF NOT EXISTS intraday_respiration (
            user_id TEXT NOT NULL, date TEXT NOT NULL,
            ts_ms INTEGER NOT NULL, value REAL NOT NULL,
            PRIMARY KEY (user_id, date, ts_ms)
        ) WITHOUT ROWID;

        CREATE TABLE IF NOT EXISTS intraday_hrv (
            user_id TEXT NOT NULL, date TEXT NOT NULL,
            ts_ms INTEGER NOT NULL, hrv_value REAL NOT NULL,
            PRIMARY KEY (user_id, date, ts_ms)
        ) WITHOUT ROWID;

        CREATE TABLE IF NOT EXISTS intraday_sleep (
            user_id TEXT NOT NULL, date TEXT NOT NULL,
            ts_ms INTEGER NOT NULL,
            stage TEXT, hr INTEGER, spo2 REAL, respiration REAL, movement REAL,
            PRIMARY KEY (user_id, date, ts_ms)
        ) WITHOUT ROWID;

        -- Extended daily fields
        CREATE TABLE IF NOT EXISTS daily_extended (
            user_id TEXT NOT NULL, date TEXT NOT NULL,
            fitness_age INTEGER, race_5k_secs REAL, race_10k_secs REAL,
            race_half_secs REAL, race_marathon_secs REAL,
            hydration_intake_ml INTEGER, hydration_goal_ml INTEGER,
            systolic_bp INTEGER, diastolic_bp INTEGER,
            training_status_phase TEXT, acute_training_load REAL,
            low_stress_secs INTEGER, medium_stress_secs INTEGER,
            high_stress_secs INTEGER, rest_stress_secs INTEGER,
            sedentary_secs INTEGER, active_secs INTEGER, highly_active_secs INTEGER,
            synced_at REAL NOT NULL,
            PRIMARY KEY (user_id, date)
        );

        -- Activity GPS tracks
        CREATE TABLE IF NOT EXISTS activity_gps_tracks (
            activity_id INTEGER NOT NULL, ts_ms INTEGER NOT NULL,
            user_id TEXT NOT NULL, date TEXT NOT NULL,
            lat REAL NOT NULL, lon REAL NOT NULL,
            altitude_m REAL, speed_mps REAL, hr INTEGER, cadence INTEGER, power_w INTEGER,
            PRIMARY KEY (activity_id, ts_ms)
        ) WITHOUT ROWID;
        CREATE INDEX IF NOT EXISTS idx_gps_user_date ON activity_gps_tracks(user_id, date);

        -- Normalized exercise sets
        CREATE TABLE IF NOT EXISTS activity_exercises (
            activity_id INTEGER NOT NULL, exercise_name TEXT NOT NULL, set_number INTEGER NOT NULL,
            user_id TEXT NOT NULL, date TEXT NOT NULL,
            reps INTEGER, weight_kg REAL, duration_secs REAL,
            PRIMARY KEY (activity_id, exercise_name, set_number)
        ) WITHOUT ROWID;
        CREATE INDEX IF NOT EXISTS idx_exercises_user_date ON activity_exercises(user_id, date);

        CREATE INDEX IF NOT EXISTS idx_daily_synced ON garmin_daily_data(user_id, synced_at);
        CREATE INDEX IF NOT EXISTS idx_users_status ON garmin_users(status);"
    ).expect("Failed to create tables");

    tracing::info!("SQLite database initialized at {}", db_path);
    pool
}
