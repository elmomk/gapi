/// Garmin Connect SSO authentication
/// Based on the garth library (https://github.com/matin/garth)
///
/// Flow:
///   1. GET /sso/embed -> cookies
///   2. GET /sso/signin -> CSRF token
///   3. POST /sso/signin -> email + password + CSRF
///   4. If MFA -> POST /sso/verifyMFA/loginEnterMfaCode
///   5. Parse ticket from response
///   6. Exchange ticket -> OAuth1 token
///   7. Exchange OAuth1 -> OAuth2 token
///   8. Use OAuth2 Bearer token for Connect API calls
mod auth;
mod api;

pub use auth::{garmin_login, garmin_submit_mfa, refresh_oauth2_token};
pub use api::{get_display_name, fetch_all_daily_data, fetch_activity_gps_track};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub const SSO_BASE: &str = "https://sso.garmin.com/sso";
pub const CONNECT_API: &str = "https://connectapi.garmin.com";
pub const OAUTH_CONSUMER_URL: &str = "https://thegarth.s3.amazonaws.com/oauth_consumer.json";
pub const USER_AGENT: &str = "com.garmin.android.apps.connectmobile";

pub static RE_CSRF: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"name="_csrf"\s+value="([^"]+)""#).unwrap()
});
pub static RE_TITLE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"<title>([^<]+)</title>").unwrap()
});
pub static RE_TICKET: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"embed\?ticket=([^"]+)"#).unwrap()
});

/// Cached OAuth consumer keys (static, fetched once from S3)
pub static OAUTH_CONSUMER: tokio::sync::OnceCell<OAuthConsumer> = tokio::sync::OnceCell::const_new();

pub async fn get_oauth_consumer(client: &Client) -> Result<&'static OAuthConsumer, String> {
    OAUTH_CONSUMER.get_or_try_init(|| async {
        client.get(OAUTH_CONSUMER_URL)
            .send().await.map_err(|e| format!("Failed to fetch OAuth consumer: {}", e))?
            .json().await.map_err(|e| format!("Failed to parse OAuth consumer: {}", e))
    }).await
}

/// Structured error type for Garmin API calls
#[derive(Debug)]
pub enum GarminApiError {
    /// HTTP 429 -- caller should stop all API calls immediately
    RateLimited,
    /// HTTP 500+ -- transient server error
    ServerError(u16),
    /// HTTP 401 -- token expired or invalid
    AuthFailed,
    /// Network-level failure (DNS, connect timeout, connection refused)
    NetworkError(String),
    /// Any other failure
    Other(String),
}

impl GarminApiError {
    pub fn is_network_error(&self) -> bool {
        matches!(self, Self::NetworkError(_))
    }
}

impl std::fmt::Display for GarminApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited => write!(f, "Rate limited (429)"),
            Self::ServerError(code) => write!(f, "Server error ({})", code),
            Self::AuthFailed => write!(f, "Authentication failed (401)"),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarminOAuth2Token {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarminSession {
    pub oauth2: GarminOAuth2Token,
    pub oauth1_token: String,
    pub oauth1_token_secret: String,
    /// Unix timestamp when the OAuth2 token was obtained (for proactive refresh)
    #[serde(default)]
    pub oauth2_created_at: i64,
}

impl GarminSession {
    /// Returns true if the OAuth2 access token is expired or will expire within 60 seconds.
    pub fn is_oauth2_expired(&self) -> bool {
        if self.oauth2_created_at == 0 {
            return false;
        }
        let now = chrono::Utc::now().timestamp();
        let expires_at = self.oauth2_created_at + self.oauth2.expires_in;
        now >= (expires_at - 60)
    }

    /// Returns true if we have OAuth1 credentials needed for token refresh.
    pub fn has_oauth1_creds(&self) -> bool {
        !self.oauth1_token.is_empty() && !self.oauth1_token_secret.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConsumer {
    pub consumer_key: String,
    pub consumer_secret: String,
}

#[derive(Debug)]
pub enum LoginResult {
    Success(GarminSession),
    MfaRequired { csrf_token: String, cookies: String },
    Error(String),
}
