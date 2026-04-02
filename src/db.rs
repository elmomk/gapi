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
        );"
    ).expect("Failed to create tables");

    tracing::info!("SQLite database initialized at {}", db_path);
    pool
}
