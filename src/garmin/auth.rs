use reqwest::Client;
use std::collections::HashMap;
use super::{
    SSO_BASE, CONNECT_API, USER_AGENT,
    RE_CSRF, RE_TITLE, RE_TICKET,
    GarminSession, GarminOAuth2Token, LoginResult,
    get_oauth_consumer,
};

/// Extract CSRF token from HTML: name="_csrf" value="..."
fn extract_csrf(html: &str) -> Option<String> {
    RE_CSRF.captures(html).map(|c| c[1].to_string())
}

/// Extract title from HTML
fn extract_title(html: &str) -> Option<String> {
    RE_TITLE.captures(html).map(|c| c[1].to_string())
}

/// Detect MFA page by checking HTML body for common MFA form indicators
fn is_mfa_page(html: &str) -> bool {
    let lower = html.to_lowercase();
    lower.contains("verification-code") || lower.contains("mfa-code")
        || lower.contains("verificationcode") || lower.contains("id=\"mfa")
        || (lower.contains("verify") && lower.contains("code"))
}

/// Check if a page title indicates an MFA/authentication challenge page
fn is_mfa_title(title: &str) -> bool {
    let lower = title.to_lowercase();
    lower.contains("mfa") || lower.contains("authentication application")
        || lower.contains("verification") || lower.contains("two-factor")
        || lower.contains("2fa")
}

/// Extract ticket from response HTML: embed?ticket=...
fn extract_ticket(html: &str) -> Option<String> {
    RE_TICKET.captures(html).map(|c| c[1].to_string())
}

/// Build the common SSO embed params
fn sso_embed_params() -> Vec<(&'static str, &'static str)> {
    vec![
        ("id", "gauth-widget"),
        ("embedWidget", "true"),
        ("gauthHost", SSO_BASE),
    ]
}

fn signin_params() -> Vec<(&'static str, String)> {
    let sso_embed = format!("{}/embed", SSO_BASE);
    vec![
        ("id", "gauth-widget".to_string()),
        ("embedWidget", "true".to_string()),
        ("gauthHost", sso_embed.clone()),
        ("service", sso_embed.clone()),
        ("source", sso_embed.clone()),
        ("redirectAfterAccountLoginUrl", sso_embed.clone()),
        ("redirectAfterAccountCreationUrl", sso_embed),
    ]
}

/// Build an SSO HTTP client with proper timeouts
fn build_sso_client() -> Result<Client, String> {
    Client::builder()
        .cookie_store(true)
        .user_agent(USER_AGENT)
        .redirect(reqwest::redirect::Policy::limited(10))
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// Send a GET request with up to 3 retries on network/DNS errors
async fn get_with_retry(client: &Client, url: &str, query: &[(&str, &str)]) -> Result<reqwest::Response, String> {
    let mut last_err = String::new();
    for attempt in 1..=3 {
        match client.get(url).query(query).send().await {
            Ok(resp) => return Ok(resp),
            Err(e) => {
                last_err = format!("{}", e);
                tracing::warn!("Request to {} failed (attempt {}/3): {}", url, attempt, last_err);
                if attempt < 3 {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    }
    Err(last_err)
}

/// Full Garmin SSO login flow. Returns a LoginResult.
pub async fn garmin_login(
    email: &str,
    password: &str,
) -> LoginResult {
    let client = match build_sso_client() {
        Ok(c) => c,
        Err(e) => return LoginResult::Error(e),
    };

    // Step 1: GET /sso/embed to set cookies
    let embed_url = format!("{}/embed", SSO_BASE);
    let embed_params: Vec<(&str, &str)> = sso_embed_params().into_iter().collect();
    if let Err(e) = get_with_retry(&client, &embed_url, &embed_params).await {
        return LoginResult::Error(format!("SSO embed request failed: {}", e));
    }

    // Step 2: GET /sso/signin to get CSRF token
    let signin_url = format!("{}/signin", SSO_BASE);
    let params = signin_params();
    let signin_resp = match client
        .get(&signin_url)
        .query(&params)
        .header("Referer", &embed_url)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return LoginResult::Error(format!("SSO signin GET failed: {}", e)),
    };

    let signin_html = match signin_resp.text().await {
        Ok(t) => t,
        Err(e) => return LoginResult::Error(format!("Failed to read signin HTML: {}", e)),
    };

    let csrf_token = match extract_csrf(&signin_html) {
        Some(t) => t,
        None => return LoginResult::Error("Could not find CSRF token in signin page".to_string()),
    };

    // Step 3: POST /sso/signin with credentials
    let mut form_data = HashMap::new();
    form_data.insert("username", email.to_string());
    form_data.insert("password", password.to_string());
    form_data.insert("embed", "true".to_string());
    form_data.insert("_csrf", csrf_token.clone());

    let signin_post_resp = match client
        .post(&signin_url)
        .query(&signin_params())
        .header("Referer", &signin_url)
        .form(&form_data)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return LoginResult::Error(format!("SSO signin POST failed: {}", e)),
    };

    let resp_html = match signin_post_resp.text().await {
        Ok(t) => t,
        Err(e) => return LoginResult::Error(format!("Failed to read signin response: {}", e)),
    };

    let title = extract_title(&resp_html).unwrap_or_default();

    // Step 4: Check for MFA
    if is_mfa_title(&title) || is_mfa_page(&resp_html) {
        tracing::info!("garmin_login: MFA/authentication page detected (title='{}')", title);
        let new_csrf = extract_csrf(&resp_html).unwrap_or(csrf_token);
        return LoginResult::MfaRequired {
            csrf_token: new_csrf,
            cookies: String::new(),
        };
    }

    // Step 5: Check for success and extract ticket
    if title != "Success" {
        tracing::warn!("garmin_login: unexpected page title='{}', html length={}", title, resp_html.len());
        return LoginResult::Error(format!(
            "Login failed. Garmin SSO returned: '{}'",
            title
        ));
    }

    complete_login(&client, &resp_html).await
}

/// Complete login after MFA or successful credential auth
async fn complete_login(client: &Client, html: &str) -> LoginResult {
    let ticket = match extract_ticket(html) {
        Some(t) => t,
        None => return LoginResult::Error("Could not find ticket in SSO response".to_string()),
    };

    // Step 6: Get OAuth consumer keys
    let consumer = match get_oauth_consumer(client).await {
        Ok(c) => c,
        Err(e) => return LoginResult::Error(e),
    };

    // Step 7: Get OAuth1 token using the ticket
    let login_url = format!("{}/embed", SSO_BASE);
    let preauth_base_url = format!(
        "{}/oauth-service/oauth/preauthorized",
        CONNECT_API
    );

    let preauth_timestamp = chrono::Utc::now().timestamp().to_string();
    let preauth_nonce = uuid::Uuid::new_v4().to_string().replace('-', "");

    let query_params = vec![
        ("ticket", ticket.as_str()),
        ("login-url", login_url.as_str()),
        ("accepts-mfa-tokens", "true"),
    ];

    let preauth_auth = build_oauth1_header_with_params(
        "GET",
        &preauth_base_url,
        &consumer.consumer_key,
        &consumer.consumer_secret,
        "",
        "",
        &preauth_timestamp,
        &preauth_nonce,
        &query_params,
    );

    let preauth_resp = match client
        .get(&preauth_base_url)
        .query(&query_params)
        .header("Authorization", &preauth_auth)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return LoginResult::Error(format!("OAuth1 preauthorized request failed: {}", e)),
    };

    let preauth_body = match preauth_resp.text().await {
        Ok(t) => t,
        Err(e) => return LoginResult::Error(format!("Failed to read preauth response: {}", e)),
    };

    let oauth1_params: HashMap<String, String> = url::form_urlencoded::parse(preauth_body.as_bytes())
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let oauth1_token = match oauth1_params.get("oauth_token") {
        Some(t) => t.clone(),
        None => return LoginResult::Error(format!(
            "No oauth_token in preauth response: {}",
            if preauth_body.len() > 200 { &preauth_body[..200] } else { &preauth_body }
        )),
    };
    let oauth1_secret = match oauth1_params.get("oauth_token_secret") {
        Some(t) => t.clone(),
        None => return LoginResult::Error("No oauth_token_secret in preauth response".to_string()),
    };
    let mfa_token = oauth1_params.get("mfa_token").cloned();

    // Step 8: Exchange OAuth1 -> OAuth2
    let exchange_url = format!("{}/oauth-service/oauth/exchange/user/2.0", CONNECT_API);

    let mut exchange_params: Vec<(&str, &str)> = Vec::new();
    if let Some(ref mfa_t) = mfa_token {
        exchange_params.push(("mfa_token", mfa_t.as_str()));
    }

    let timestamp = chrono::Utc::now().timestamp().to_string();
    let nonce = uuid::Uuid::new_v4().to_string().replace('-', "");

    let auth_header = build_oauth1_header_with_params(
        "POST",
        &exchange_url,
        &consumer.consumer_key,
        &consumer.consumer_secret,
        &oauth1_token,
        &oauth1_secret,
        &timestamp,
        &nonce,
        &exchange_params,
    );

    let mut exchange_form = HashMap::new();
    if let Some(ref mfa_t) = mfa_token {
        exchange_form.insert("mfa_token", mfa_t.as_str());
    }

    let exchange_resp = match client
        .post(&exchange_url)
        .header("Authorization", &auth_header)
        .header("User-Agent", USER_AGENT)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&exchange_form)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return LoginResult::Error(format!("OAuth2 exchange request failed: {}", e)),
    };

    if !exchange_resp.status().is_success() {
        let status = exchange_resp.status();
        let body = exchange_resp.text().await.unwrap_or_default();
        return LoginResult::Error(format!(
            "OAuth2 exchange failed ({}): {}",
            status,
            if body.len() > 200 { &body[..200] } else { &body }
        ));
    }

    match exchange_resp.json::<GarminOAuth2Token>().await {
        Ok(token) => LoginResult::Success(GarminSession {
            oauth2: token,
            oauth1_token: oauth1_token.clone(),
            oauth1_token_secret: oauth1_secret.clone(),
            oauth2_created_at: chrono::Utc::now().timestamp(),
        }),
        Err(e) => LoginResult::Error(format!("Failed to parse OAuth2 token: {}", e)),
    }
}

/// Build an OAuth1 Authorization header, including additional query/body params in the signature
#[allow(clippy::too_many_arguments)]
fn build_oauth1_header_with_params(
    method: &str,
    url: &str,
    consumer_key: &str,
    consumer_secret: &str,
    token: &str,
    token_secret: &str,
    timestamp: &str,
    nonce: &str,
    extra_params: &[(&str, &str)],
) -> String {
    use base64::prelude::*;
    use hmac::{Hmac, Mac};
    use sha1::Sha1;

    let mut params: Vec<(&str, &str)> = vec![
        ("oauth_consumer_key", consumer_key),
        ("oauth_nonce", nonce),
        ("oauth_signature_method", "HMAC-SHA1"),
        ("oauth_timestamp", timestamp),
        ("oauth_version", "1.0"),
    ];
    if !token.is_empty() {
        params.push(("oauth_token", token));
    }
    for &(k, v) in extra_params {
        params.push((k, v));
    }
    params.sort_by_key(|&(k, _)| k);

    let param_string: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let base_string = format!(
        "{}&{}&{}",
        method.to_uppercase(),
        percent_encode(url),
        percent_encode(&param_string)
    );

    let signing_key = format!("{}&{}", percent_encode(consumer_secret), percent_encode(token_secret));

    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(base_string.as_bytes());
    let signature = BASE64_STANDARD.encode(mac.finalize().into_bytes());

    let mut header = format!(
        r#"OAuth oauth_consumer_key="{}", oauth_nonce="{}", oauth_signature="{}", oauth_signature_method="HMAC-SHA1", oauth_timestamp="{}""#,
        percent_encode(consumer_key),
        percent_encode(nonce),
        percent_encode(&signature),
        percent_encode(timestamp),
    );
    if !token.is_empty() {
        header.push_str(&format!(r#", oauth_token="{}""#, percent_encode(token)));
    }
    header.push_str(r#", oauth_version="1.0""#);
    header
}

/// Percent-encode per RFC 5849
fn percent_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// Submit MFA code and complete login
pub async fn garmin_submit_mfa(
    email: &str,
    password: &str,
    mfa_code: &str,
) -> LoginResult {
    let client = match build_sso_client() {
        Ok(c) => c,
        Err(e) => return LoginResult::Error(e),
    };

    // Re-do the full login flow up to MFA
    let embed_url = format!("{}/embed", SSO_BASE);
    let embed_params: Vec<(&str, &str)> = sso_embed_params().into_iter().collect();
    if let Err(e) = get_with_retry(&client, &embed_url, &embed_params).await {
        return LoginResult::Error(format!("SSO embed request failed: {}", e));
    }

    let signin_url = format!("{}/signin", SSO_BASE);
    let params = signin_params();
    let signin_resp = match client
        .get(&signin_url)
        .query(&params)
        .header("Referer", &embed_url)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return LoginResult::Error(format!("SSO signin GET failed: {}", e)),
    };

    let signin_html = match signin_resp.text().await {
        Ok(t) => t,
        Err(e) => return LoginResult::Error(format!("Failed to read signin HTML: {}", e)),
    };

    let csrf_token = match extract_csrf(&signin_html) {
        Some(t) => t,
        None => return LoginResult::Error("Could not find CSRF token".to_string()),
    };

    let mut form_data = HashMap::new();
    form_data.insert("username", email.to_string());
    form_data.insert("password", password.to_string());
    form_data.insert("embed", "true".to_string());
    form_data.insert("_csrf", csrf_token);

    let signin_post_resp = match client
        .post(&signin_url)
        .query(&signin_params())
        .header("Referer", &signin_url)
        .form(&form_data)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return LoginResult::Error(format!("SSO signin POST failed: {}", e)),
    };

    let resp_html = match signin_post_resp.text().await {
        Ok(t) => t,
        Err(e) => return LoginResult::Error(format!("Failed to read signin response: {}", e)),
    };

    let title = extract_title(&resp_html).unwrap_or_default();
    if !is_mfa_title(&title) && !is_mfa_page(&resp_html) {
        if title == "Success" {
            return complete_login(&client, &resp_html).await;
        }
        return LoginResult::Error(format!("Expected MFA page, got: '{}'", title));
    }

    // Submit MFA code
    let mfa_csrf = extract_csrf(&resp_html).unwrap_or_default();
    let mfa_url = format!("{}/verifyMFA/loginEnterMfaCode", SSO_BASE);

    let mut mfa_form = HashMap::new();
    mfa_form.insert("mfa-code", mfa_code.to_string());
    mfa_form.insert("embed", "true".to_string());
    mfa_form.insert("_csrf", mfa_csrf);
    mfa_form.insert("fromPage", "setupEnterMfaCode".to_string());

    let mfa_resp = match client
        .post(&mfa_url)
        .query(&signin_params())
        .header("Referer", &signin_url)
        .form(&mfa_form)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return LoginResult::Error(format!("MFA verify POST failed: {}", e)),
    };

    let mfa_html = match mfa_resp.text().await {
        Ok(t) => t,
        Err(e) => return LoginResult::Error(format!("Failed to read MFA response: {}", e)),
    };

    let mfa_title = extract_title(&mfa_html).unwrap_or_default();
    if mfa_title != "Success" {
        return LoginResult::Error(format!("MFA verification failed: '{}'", mfa_title));
    }

    complete_login(&client, &mfa_html).await
}

/// Refresh an OAuth2 token using OAuth1 credentials.
pub async fn refresh_oauth2_token(
    client: &Client,
    session: &GarminSession,
) -> Result<GarminSession, String> {
    let consumer = get_oauth_consumer(client).await?;

    let exchange_url = format!("{}/oauth-service/oauth/exchange/user/2.0", CONNECT_API);
    let timestamp = chrono::Utc::now().timestamp().to_string();
    let nonce = uuid::Uuid::new_v4().to_string().replace('-', "");

    let auth_header = build_oauth1_header_with_params(
        "POST",
        &exchange_url,
        &consumer.consumer_key,
        &consumer.consumer_secret,
        &session.oauth1_token,
        &session.oauth1_token_secret,
        &timestamp,
        &nonce,
        &[],
    );

    let resp = client
        .post(&exchange_url)
        .header("Authorization", &auth_header)
        .header("User-Agent", USER_AGENT)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .map_err(|e| format!("OAuth2 refresh request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("OAuth2 refresh failed ({}): {}", status, &body[..body.len().min(200)]));
    }

    let new_token = resp.json::<GarminOAuth2Token>().await
        .map_err(|e| format!("Failed to parse refreshed OAuth2 token: {}", e))?;

    Ok(GarminSession {
        oauth2: new_token,
        oauth1_token: session.oauth1_token.clone(),
        oauth1_token_secret: session.oauth1_token_secret.clone(),
        oauth2_created_at: chrono::Utc::now().timestamp(),
    })
}
