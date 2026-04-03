/// Application configuration loaded from environment variables
pub struct AppConfig {
    pub database_path: String,
    pub master_key: String,
    pub host: String,
    pub port: u16,
    pub sync_rate_limit_mins: i64,
    pub sync_days: i64,
    pub garmin_api_delay_secs: u64,
    pub max_consecutive_empty_days: u64,
    pub sync_intraday: bool,
    pub sync_intraday_days: i64,
    pub sync_gps_tracks: bool,
    pub intraday_retention_days: i64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database_path: std::env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "garmin_api.db".to_string()),
            master_key: std::env::var("MASTER_KEY")
                .expect("MASTER_KEY must be set (at least 32 bytes)"),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a valid u16"),
            sync_rate_limit_mins: std::env::var("SYNC_RATE_LIMIT_MINS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            sync_days: std::env::var("SYNC_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            garmin_api_delay_secs: std::env::var("GARMIN_API_DELAY_SECS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            max_consecutive_empty_days: std::env::var("MAX_CONSECUTIVE_EMPTY_DAYS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            sync_intraday: std::env::var("SYNC_INTRADAY")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            sync_intraday_days: std::env::var("SYNC_INTRADAY_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .unwrap_or(7),
            sync_gps_tracks: std::env::var("SYNC_GPS_TRACKS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            intraday_retention_days: std::env::var("INTRADAY_RETENTION_DAYS")
                .unwrap_or_else(|_| "90".to_string())
                .parse()
                .unwrap_or(90),
        }
    }
}
