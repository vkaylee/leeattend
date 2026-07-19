//! Auth cookie helpers — building Set-Cookie headers for access/refresh tokens.
//!
//! Extracted from handlers/auth to keep handler files focused on request/response glue.

use axum::http::{header::SET_COOKIE, HeaderMap, HeaderValue};

/// Build Set-Cookie headers for access_token and refresh_token (HttpOnly + Secure).
pub fn create_auth_cookies(access_token: &str, refresh_token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let domain_attr = cookie_domain_attr();

    let access_cookie = format!(
        "access_token={access_token}; {domain_attr}HttpOnly; SameSite=None; Secure; Path=/; Max-Age=900"
    );
    let refresh_cookie = format!(
        "refresh_token={refresh_token}; {domain_attr}HttpOnly; SameSite=None; Secure; Path=/; Max-Age=2592000"
    );

    if let Ok(val) = HeaderValue::from_str(&access_cookie) {
        headers.append(SET_COOKIE, val);
    }
    if let Ok(val) = HeaderValue::from_str(&refresh_cookie) {
        headers.append(SET_COOKIE, val);
    }

    headers
}

/// Build Set-Cookie headers that clear (expire) both auth cookies.
pub fn clear_auth_cookies() -> HeaderMap {
    let mut headers = HeaderMap::new();
    let domain_attr = cookie_domain_attr();

    let access_cookie =
        format!("access_token=; {domain_attr}HttpOnly; SameSite=None; Secure; Path=/; Max-Age=0");
    let refresh_cookie =
        format!("refresh_token=; {domain_attr}HttpOnly; SameSite=None; Secure; Path=/; Max-Age=0");

    if let Ok(val) = HeaderValue::from_str(&access_cookie) {
        headers.append(SET_COOKIE, val);
    }
    if let Ok(val) = HeaderValue::from_str(&refresh_cookie) {
        headers.append(SET_COOKIE, val);
    }

    headers
}

fn cookie_domain_attr() -> String {
    match std::env::var("COOKIE_DOMAIN") {
        Ok(d) if !d.is_empty() => format!("Domain={d}; "),
        _ => String::new(),
    }
}
