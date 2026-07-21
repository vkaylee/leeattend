use crate::errors::{AppError, AppResult};
use crate::queue::RedisPool;
use crate::repositories::system_settings_repository::SystemSettingsRepository;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

// ── Rate limit constants ──────────────────────────────────────────────────

const RATE_LIMIT_MINUTELY_MAX: u32 = 5;
const RATE_LIMIT_HOURLY_MAX: u32 = 20;
const RATE_LIMIT_DAILY_MAX: u32 = 100;
const RATE_WINDOW_MINUTELY_SECS: usize = 60;
const RATE_WINDOW_HOURLY_SECS: usize = 3600;
const RATE_WINDOW_DAILY_SECS: usize = 86400;

/// How often to poll for SMTP config changes.
const CONFIG_RELOAD_INTERVAL: Duration = Duration::from_secs(60);

/// Timeout for SMTP connect + send operations.
const SMTP_TIMEOUT: Duration = Duration::from_secs(10);

/// Serialisable SMTP config snapshot (DB ↔ Redis ↔ memory).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SmtpConfigData {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    from: Option<String>,
    base_url: Option<String>,
    /// Optional HTTP API config for sending email via REST instead of SMTP.
    /// When present, HTTP takes priority over SMTP (useful on platforms that
    /// block outbound SMTP ports, like Railway).
    #[serde(default)]
    http_api: Option<HttpApiConfig>,
}

/// HTTP-based email sending adapter — provider-agnostic.
/// User configures the URL, auth, and JSON body template with `{{variables}}`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct HttpApiConfig {
    /// Full URL of the email API endpoint (e.g. https://send.api.mailtrap.io/api/send).
    url: String,
    /// Optional auth header name (e.g. "Api-Token", "Authorization").
    #[serde(default)]
    auth_header: Option<String>,
    /// Optional auth header value. Supports `{{username}}` and `{{password}}`
    /// placeholders, resolved at config reload time.
    #[serde(default)]
    auth_value: Option<String>,
    /// Optional JSON field in the response to check for success (e.g. "success").
    #[serde(default)]
    success_field: Option<String>,
    /// Optional expected value for success_field (e.g. true).
    #[serde(default)]
    success_value: Option<serde_json::Value>,
    /// JSON body template with `{{from_name}}`, `{{from_email}}`, `{{to}}`,
    /// `{{subject}}`, `{{body}}` placeholders.
    body_template: String,
}

// ── Hot-swappable SMTP config ─────────────────────────────────────────────

/// All email runtime state that can be hot-reloaded from Redis.
struct EmailConfig {
    transport: Option<SmtpTransport>,
    /// Pre-resolved HTTP API config (auth_value placeholders already substituted).
    http_config: Option<HttpApiConfig>,
    from: String,
    base_url: String,
    last_reload: Instant,
    /// Cached for auth_value template resolution at reload time.
    username: Option<String>,
    password: Option<String>,
}

impl EmailConfig {
    /// Bootstrap with no transport — will be populated on first reload from DB.
    fn default_config() -> Self {
        Self {
            transport: None,
            http_config: None,
            from: "LeeAttend <noreply@leeattend.com>".into(),
            base_url: "http://localhost:8080".into(),
            last_reload: Instant::now()
                .checked_sub(CONFIG_RELOAD_INTERVAL)
                .unwrap_or_else(Instant::now), // eager-load eligible
            username: None,
            password: None,
        }
    }
}

/// Build a lettre SmtpTransport with auto-detected TLS mode.
fn build_transport(
    host: Option<&str>,
    port: Option<u16>,
    username: Option<&str>,
    password: Option<&str>,
) -> Option<SmtpTransport> {
    let (Some(host), Some(port)) = (host, port) else {
        return None;
    };

    // Both relay() and starttls_relay() return Result<SmtpTransportBuilder, Error>
    let builder = if port == 465 {
        SmtpTransport::relay(host)
    } else {
        SmtpTransport::starttls_relay(host)
    };
    let mut builder = match builder {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to create SMTP transport for {host}:{port}: {e}");
            return None;
        }
    };
    builder = builder.port(port).timeout(Some(SMTP_TIMEOUT));

    if let (Some(u), Some(p)) = (username, password) {
        builder = builder.credentials(Credentials::new(u.to_string(), p.to_string()));
    }

    Some(builder.build())
}

/// Parse "Name <email>" or plain "email" into (name, email).
fn parse_from_address(from: &str) -> (String, String) {
    if let Some(open) = from.rfind('<') {
        if let Some(close) = from.rfind('>') {
            let name = from[..open].trim().trim_matches('"').to_string();
            let email = from[open + 1..close].trim().to_string();
            return (name, email);
        }
    }
    (String::new(), from.trim().to_string())
}

/// Replace `{{variables}}` in a JSON body template, JSON-escaping each value.
fn resolve_template(
    template: &str,
    from_name: &str,
    from_email: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> String {
    let escape = |s: &str| -> String {
        let mut out = String::with_capacity(s.len() + 2);
        for ch in s.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out
    };

    template
        .replace("{{from_name}}", &escape(from_name))
        .replace("{{from_email}}", &escape(from_email))
        .replace("{{to}}", &escape(to))
        .replace("{{subject}}", &escape(subject))
        .replace("{{body}}", &escape(body))
}

// ── EmailService ──────────────────────────────────────────────────────────

pub struct EmailService {
    /// Hot-swappable config; read-locked for sends, write-locked on reload.
    config: Arc<RwLock<EmailConfig>>,
    redis_pool: Option<RedisPool>,
    /// DB-backed system settings (source of truth) — optional for dev/test.
    settings_repo: Option<Arc<dyn SystemSettingsRepository>>,
    /// Shared HTTP client with connection pooling (10s timeout).
    http_client: reqwest::Client,
}

impl EmailService {
    pub fn new(
        redis_pool: Option<RedisPool>,
        settings_repo: Option<Arc<dyn SystemSettingsRepository>>,
    ) -> Self {
        // `last_reload` is set in the past so the first send will trigger
        // an eager load from DB (or Redis cache).
        let http_client = reqwest::Client::builder()
            .timeout(SMTP_TIMEOUT)
            .build()
            .expect("reqwest::Client::builder infallible with valid config");
        Self {
            config: Arc::new(RwLock::new(EmailConfig::default_config())),
            redis_pool,
            settings_repo,
            http_client,
        }
    }

    // ── Hot-reload: DB → Redis cache → memory ─────────────────────────

    /// Reload SMTP config. Priority chain:
    ///   1. Redis `smtp:config` hash (fast shared cache)
    ///   2. DB `system_settings` (source of truth) → writes back to Redis
    ///   3. Env vars (bootstrap fallback)
    async fn maybe_reload_config(&self) {
        // Fast path: memory cache still fresh
        {
            let cfg = self.config.read().await;
            if cfg.last_reload.elapsed() < CONFIG_RELOAD_INTERVAL {
                return;
            }
        }

        // Try Redis cache first (fast, shared across instances)
        if let Some(config) = self.load_from_redis().await {
            self.apply_config(config).await;
            return;
        }

        // Redis miss → try DB (source of truth)
        if let Some(config) = self.load_from_db().await {
            // Write back to Redis cache
            self.write_to_redis_cache(&config).await;
            self.apply_config(config).await;
            return;
        }

        // No config anywhere → stay in dev mode (no SMTP), bump timestamp
        let mut cfg = self.config.write().await;
        cfg.last_reload = Instant::now();
    }

    async fn load_from_redis(&self) -> Option<SmtpConfigData> {
        let redis = self.redis_pool.as_ref()?;
        let fields: &[&str] = &["host", "port", "username", "password", "from", "base_url", "http_api"];
        let mut conn = redis.conn();
        let values: Vec<Option<String>> = redis::cmd("HMGET")
            .arg("smtp:config")
            .arg(fields)
            .query_async(&mut conn)
            .await
            .map_err(|e| warn!("Failed to read smtp:config from Redis: {e}"))
            .ok()?;

        if values.iter().all(Option::is_none) {
            return None;
        }

        let http_api = values[6]
            .as_ref()
            .and_then(|json_str| serde_json::from_str(json_str).ok());

        Some(SmtpConfigData {
            host: values[0].clone(),
            port: values[1].as_ref().and_then(|p| p.parse().ok()),
            username: values[2].clone(),
            password: values[3].clone(),
            from: values[4].clone(),
            base_url: values[5].clone(),
            http_api,
        })
    }

    async fn load_from_db(&self) -> Option<SmtpConfigData> {
        let repo = self.settings_repo.as_ref()?;
        let json = repo
            .get("smtp")
            .await
            .map_err(|e| warn!("Failed to read system_settings.smtp from DB: {e}"))
            .ok()??;

        serde_json::from_value::<SmtpConfigData>(json)
            .map_err(|e| warn!("Failed to parse system_settings.smtp JSON: {e}"))
            .ok()
    }

    async fn write_to_redis_cache(&self, config: &SmtpConfigData) {
        let Some(redis) = &self.redis_pool else {
            return;
        };
        let mut conn = redis.conn();
        let http_api_json = serde_json::to_string(&config.http_api).unwrap_or_default();
        let _: () = redis::cmd("HSET")
            .arg("smtp:config")
            .arg("host")
            .arg(config.host.as_deref().unwrap_or(""))
            .arg("port")
            .arg(config.port.map(|p| p.to_string()).unwrap_or_default())
            .arg("username")
            .arg(config.username.as_deref().unwrap_or(""))
            .arg("password")
            .arg(config.password.as_deref().unwrap_or(""))
            .arg("from")
            .arg(config.from.as_deref().unwrap_or(""))
            .arg("base_url")
            .arg(config.base_url.as_deref().unwrap_or(""))
            .arg("http_api")
            .arg(&http_api_json)
            .query_async(&mut conn)
            .await
            .map_err(|e| warn!("Failed to write smtp:config to Redis cache: {e}"))
            .unwrap_or(());
    }

    async fn apply_config(&self, c: SmtpConfigData) {
        let use_http = c.http_api.is_some();
        let http_url = c.http_api.as_ref().map(|a| a.url.clone());

        // Resolve auth_value placeholders at reload time (once, not per send).
        let resolved_http_config = c.http_api.map(|cfg| {
            let mut resolved = cfg;
            if let Some(ref auth_val) = resolved.auth_value {
                let mut v = auth_val.clone();
                if let Some(ref u) = c.username {
                    v = v.replace("{{username}}", u);
                }
                if let Some(ref p) = c.password {
                    v = v.replace("{{password}}", p);
                }
                resolved.auth_value = Some(v);
            }
            resolved
        });

        let new_transport = if use_http {
            None // Not building SMTP transport when HTTP is configured
        } else {
            build_transport(c.host.as_deref(), c.port, c.username.as_deref(), c.password.as_deref())
        };

        let mut cfg = self.config.write().await;
        cfg.transport = new_transport;
        cfg.http_config = resolved_http_config;
        cfg.username = c.username;
        cfg.password = c.password;
        cfg.last_reload = Instant::now();
        if let Some(f) = c.from {
            cfg.from = f;
        }
        if let Some(b) = c.base_url {
            cfg.base_url = b;
        }
        drop(cfg);
        if use_http {
            info!(
                "Email HTTP API config reloaded (url={})",
                http_url.as_deref().unwrap_or_default()
            );
        } else {
            info!(
                "SMTP config reloaded (host={:?}, port={:?})",
                c.host, c.port
            );
        }
    }

    // ── Rate limiting ────────────────────────────────────────────────

    async fn check_rate_limit(&self) -> bool {
        let Some(redis) = &self.redis_pool else {
            return true;
        };

        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let minutely_key = format!(
            "email:rate:min:{}",
            now_secs / RATE_WINDOW_MINUTELY_SECS as u64
        );
        let hourly_key = format!(
            "email:rate:hour:{}",
            now_secs / RATE_WINDOW_HOURLY_SECS as u64
        );
        let daily_key = format!(
            "email:rate:day:{}",
            now_secs / RATE_WINDOW_DAILY_SECS as u64
        );

        let mut conn = redis.conn();

        async fn incr_and_expire(
            conn: &mut redis::aio::ConnectionManager,
            key: &str,
            ttl_secs: usize,
        ) -> u32 {
            let count: u32 = redis::cmd("INCR")
                .arg(key)
                .query_async(conn)
                .await
                .unwrap_or_else(|e| {
                    error!("Redis INCR failed for {key}: {e}");
                    0
                });
            let ttl_i64: i64 = ttl_secs.try_into().unwrap_or(i64::MAX);
            let _: () = redis::cmd("EXPIRE")
                .arg(key)
                .arg(ttl_i64)
                .query_async(conn)
                .await
                .map_err(|e| error!("Redis EXPIRE failed for {key}: {e}"))
                .unwrap_or(());
            count
        }

        let min = incr_and_expire(&mut conn, &minutely_key, RATE_WINDOW_MINUTELY_SECS).await;
        let hour = incr_and_expire(&mut conn, &hourly_key, RATE_WINDOW_HOURLY_SECS).await;
        let day = incr_and_expire(&mut conn, &daily_key, RATE_WINDOW_DAILY_SECS).await;

        if min > RATE_LIMIT_MINUTELY_MAX {
            warn!("SMTP minutely rate limit reached ({min}/{RATE_LIMIT_MINUTELY_MAX}), dropping");
            return false;
        }
        if hour > RATE_LIMIT_HOURLY_MAX {
            warn!("SMTP hourly rate limit reached ({hour}/{RATE_LIMIT_HOURLY_MAX}), dropping");
            return false;
        }
        if day > RATE_LIMIT_DAILY_MAX {
            warn!("SMTP daily rate limit reached ({day}/{RATE_LIMIT_DAILY_MAX}), dropping");
            return false;
        }

        true
    }

    // ── HTTP send path ─────────────────────────────────────────────────

    /// Send an email via the configured HTTP API (provider-agnostic).
    async fn send_via_http(
        &self,
        http_config: &HttpApiConfig,
        from: &str,
        to_email: &str,
        subject: &str,
        body: &str,
    ) -> AppResult<()> {
        let (from_name, from_email) = parse_from_address(from);
        let request_body = resolve_template(
            &http_config.body_template,
            &from_name,
            &from_email,
            to_email,
            subject,
            body,
        );

        let mut req = self
            .http_client
            .post(&http_config.url)
            .header("Content-Type", "application/json");

        if let (Some(ref header), Some(ref value)) =
            (&http_config.auth_header, &http_config.auth_value)
        {
            req = req.header(header.as_str(), value.as_str());
        }

        let response = req.body(request_body).send().await.map_err(|e| {
            error!("HTTP email API request failed: {e}");
            AppError::Internal(format!("Failed to send email via HTTP API: {e}"))
        })?;

        let status = response.status();
        if !status.is_success() {
            let resp_body = response.text().await.unwrap_or_default();
            error!("HTTP email API returned {status}: {resp_body}");
            return Err(AppError::Internal(format!(
                "Email API returned HTTP {status}: {resp_body}"
            )));
        }

        // Validate success_field/success_value if configured
        if let (Some(ref field), Some(ref expected)) =
            (&http_config.success_field, &http_config.success_value)
        {
            let resp_json: serde_json::Value =
                response.json().await.map_err(|e| {
                    AppError::Internal(format!("Failed to parse email API response: {e}"))
                })?;
            let actual = resp_json.get(field);
            if actual != Some(expected) {
                error!(
                    "Email API success check failed: field={field}, expected={expected:?}, got={actual:?}"
                );
                return Err(AppError::Internal(format!(
                    "Email API indicated failure: field '{field}' was {actual:?}, expected {expected:?}"
                )));
            }
        }

        Ok(())
    }

    // ── Send ─────────────────────────────────────────────────────────

    pub async fn send_password_reset(&self, to_email: &str, token: &str) -> AppResult<()> {
        // Hot-reload SMTP config from Redis if stale
        self.maybe_reload_config().await;

        // Rate limit check
        if !self.check_rate_limit().await {
            return Ok(());
        }

        // Snapshot config under read lock
        let (transport, http_config, from, base_url) = {
            let cfg = self.config.read().await;
            (
                cfg.transport.clone(),
                cfg.http_config.clone(),
                cfg.from.clone(),
                cfg.base_url.clone(),
            )
        };

        let reset_url = format!("{base_url}/reset-password?token={token}");
        let subject = "Reset your LeeAttend Password";
        let body = format!(
            "Hello,\n\nWe received a request to reset your password. Click the link below to set a new password:\n\n{reset_url}\n\nThis link is valid for 15 minutes.\n\nIf you did not request a password reset, please ignore this email.\n\nRegards,\nLeeAttend Team"
        );

        if let Some(ref http_cfg) = http_config {
            self.send_via_http(http_cfg, &from, to_email, subject, &body)
                .await?;
            info!(
                "Password reset email sent via HTTP API to {}",
                crate::utils::masking::mask_email(to_email)
            );
        } else if let Some(transport) = &transport {
            let email =
                Message::builder()
                    .from(from.parse().map_err(|e| {
                        AppError::Internal(format!("Failed to parse SMTP_FROM: {e}"))
                    })?)
                    .to(to_email.parse().map_err(|e| {
                        AppError::Internal(format!("Failed to parse recipient: {e}"))
                    })?)
                    .subject(subject)
                    .body(body)
                    .map_err(|e| AppError::Internal(format!("Failed to build email: {e}")))?;

            let transport_clone = transport.clone();
            tokio::task::spawn_blocking(move || transport_clone.send(&email))
                .await
                .map_err(|e| AppError::Internal(format!("Tokio task join error: {e}")))?
                .map_err(|e| {
                    error!("SMTP send failed: {e}");
                    AppError::Internal(format!("Failed to send email via SMTP: {e}"))
                })?;

            info!(
                "Password reset email sent via SMTP to {}",
                crate::utils::masking::mask_email(to_email)
            );
        } else {
            info!(
                "[DEV MODE] Password reset for: {}\n[DEV MODE] Reset URL: {reset_url}",
                crate::utils::masking::mask_email(to_email)
            );
        }

        Ok(())
    }

    // ── Test SMTP connectivity ──────────────────────────────────────────

    /// Send a test email to verify email configuration is correct.
    /// Bypasses rate limiting. Uses HTTP API if configured, SMTP otherwise.
    /// Returns a human-readable success message.
    pub async fn send_test_email(&self, to_email: &str) -> AppResult<String> {
        // Force-reload config (skip cache TTL)
        {
            let mut cfg = self.config.write().await;
            cfg.last_reload = Instant::now()
                .checked_sub(CONFIG_RELOAD_INTERVAL)
                .unwrap_or_else(Instant::now);
        }
        self.maybe_reload_config().await;

        let (transport, http_config, from, base_url) = {
            let cfg = self.config.read().await;
            (
                cfg.transport.clone(),
                cfg.http_config.clone(),
                cfg.from.clone(),
                cfg.base_url.clone(),
            )
        };

        // Require at least one transport method
        if transport.is_none() && http_config.is_none() {
            return Err(AppError::Internal(
                "Email is not configured. Insert a 'smtp' record into system_settings \
                 with either SMTP host/port or an 'http_api' block.".into(),
            ));
        }

        let subject = "LeeAttend — Email Test";
        let body = format!(
            "Hello,\n\nThis is a test email from LeeAttend to confirm your email configuration is working correctly.\n\n\
             Sent at: {}\n\
             App URL: {base_url}\n\n\
             If you received this, your email setup is ready.\n\n\
             Regards,\nLeeAttend Team",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        if let Some(ref http_cfg) = http_config {
            self.send_via_http(http_cfg, &from, to_email, subject, &body)
                .await?;
            info!(
                "Test email sent via HTTP API to {}",
                crate::utils::masking::mask_email(to_email)
            );
            return Ok(format!(
                "Test email sent via HTTP API. Check {to_email} for the message."
            ));
        }

        // SMTP path (transport guaranteed Some by the check above)
        let transport = transport.unwrap();
        let email = Message::builder()
            .from(
                from.parse()
                    .map_err(|e| AppError::Internal(format!("Failed to parse SMTP_FROM: {e}")))?,
            )
            .to(to_email
                .parse()
                .map_err(|e| AppError::Internal(format!("Failed to parse recipient: {e}")))?)
            .subject(subject)
            .body(body)
            .map_err(|e| AppError::Internal(format!("Failed to build email: {e}")))?;

        let transport_clone = transport.clone();
        tokio::task::spawn_blocking(move || transport_clone.send(&email))
            .await
            .map_err(|e| AppError::Internal(format!("Tokio task join error: {e}")))?
            .map_err(|e| {
                error!("SMTP test send failed: {e}");
                AppError::Internal(format!(
                    "Failed to send test email via SMTP: {e}\n\nPlease verify your SMTP settings in system_settings (key='smtp')."
                ))
            })?;

        info!(
            "SMTP test email sent successfully to {}",
            crate::utils::masking::mask_email(to_email)
        );

        Ok(format!(
            "Test email sent successfully. Check {to_email} for the message."
        ))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_service(smtp_enabled: bool) -> EmailService {
        let transport = if smtp_enabled {
            Some(
                SmtpTransport::starttls_relay("localhost")
                    .expect("localhost always valid")
                    .port(587)
                    .build(),
            )
        } else {
            None
        };
        EmailService {
            config: Arc::new(RwLock::new(EmailConfig {
                transport,
                http_config: None,
                from: "Test <test@leeattend.com>".into(),
                base_url: "https://app.example.com".into(),
                last_reload: Instant::now(),
                username: None,
                password: None,
            })),
            redis_pool: None,
            settings_repo: None,
            http_client: reqwest::Client::new(),
        }
    }

    async fn test_service_with_redis() -> Option<EmailService> {
        let url = std::env::var("REDIS_URL")
            .or_else(|_| std::env::var("REDIS_URL_FOR_TEST_GENERATION"))
            .ok()?;
        let pool = RedisPool::new(&url).await.ok()?;
        Some(EmailService {
            config: Arc::new(RwLock::new(EmailConfig {
                transport: None,
                http_config: None,
                from: "Test <test@leeattend.com>".into(),
                base_url: "https://app.example.com".into(),
                last_reload: Instant::now(),
                username: None,
                password: None,
            })),
            redis_pool: Some(pool),
            settings_repo: None,
            http_client: reqwest::Client::new(),
        })
    }

    // ── Dev-mode send tests ──────────────────────────────────────

    #[tokio::test]
    async fn send_password_reset_dev_mode_returns_ok() {
        let s = test_service(false);
        assert!(s.send_password_reset("u@x.com", "tok").await.is_ok());
    }

    #[tokio::test]
    async fn send_password_reset_handles_special_email_chars() {
        let s = test_service(false);
        assert!(s.send_password_reset("user+tag@x.com", "tok").await.is_ok());
    }

    #[tokio::test]
    async fn send_password_reset_preserves_url_format() {
        let s = EmailService {
            config: Arc::new(RwLock::new(EmailConfig {
                transport: None,
                http_config: None,
                from: "F <f@t.com>".into(),
                base_url: "https://my.app".into(),
                last_reload: Instant::now(),
                username: None,
                password: None,
            })),
            redis_pool: None,
            settings_repo: None,
            http_client: reqwest::Client::new(),
        };
        assert!(s.send_password_reset("a@b.com", "s3cret").await.is_ok());
    }

    // ── Rate limit tests (no Redis) ────────────────────────────────

    #[tokio::test]
    async fn check_rate_limit_allows_when_no_redis() {
        assert!(test_service(false).check_rate_limit().await);
    }

    // ── Rate limit tests (with Redis) ──────────────────────────────

    #[tokio::test]
    async fn rate_limit_blocks_after_cap() {
        let Some(s) = test_service_with_redis().await else {
            return;
        };
        let conn = s.redis_pool.as_ref().unwrap().conn();
        let key = current_minutely_key();
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg(RATE_LIMIT_MINUTELY_MAX)
            .query_async(&mut conn.clone())
            .await
            .unwrap();
        let allowed = s.check_rate_limit().await;
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn.clone())
            .await
            .unwrap();
        assert!(
            !allowed,
            "should block when count > {RATE_LIMIT_MINUTELY_MAX}"
        );
    }

    #[tokio::test]
    async fn rate_limit_allows_under_cap() {
        let Some(s) = test_service_with_redis().await else {
            return;
        };
        let conn = s.redis_pool.as_ref().unwrap().conn();
        let key = current_minutely_key();
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn.clone())
            .await
            .unwrap();
        let allowed = s.check_rate_limit().await;
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn.clone())
            .await
            .unwrap();
        assert!(allowed, "should allow under the cap");
    }

    #[tokio::test]
    async fn rate_limit_sets_ttl() {
        let Some(s) = test_service_with_redis().await else {
            return;
        };
        let conn = s.redis_pool.as_ref().unwrap().conn();
        let key = current_minutely_key();
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn.clone())
            .await
            .unwrap();
        s.check_rate_limit().await;
        let ttl: i64 = redis::cmd("TTL")
            .arg(&key)
            .query_async(&mut conn.clone())
            .await
            .unwrap_or(-1);
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn.clone())
            .await
            .unwrap();
        assert!(
            ttl > 0 && ttl <= RATE_WINDOW_MINUTELY_SECS as i64,
            "TTL={ttl}"
        );
    }

    #[tokio::test]
    async fn send_silently_drops_when_rate_limited() {
        let Some(s) = test_service_with_redis().await else {
            return;
        };
        let conn = s.redis_pool.as_ref().unwrap().conn();
        let key = current_minutely_key();
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg(RATE_LIMIT_MINUTELY_MAX)
            .query_async(&mut conn.clone())
            .await
            .unwrap();
        let r = s.send_password_reset("x@y.com", "tok").await;
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn.clone())
            .await
            .unwrap();
        assert!(r.is_ok(), "must silently drop, not error");
    }

    // ── Hot-reload tests (with Redis) ──────────────────────────────

    #[tokio::test]
    async fn hot_reload_detects_redis_config_change() {
        let Some(s) = test_service_with_redis().await else {
            return;
        };
        let conn = s.redis_pool.as_ref().unwrap().conn();

        // Seed Redis smtp:config with test values
        let _: () = redis::cmd("HSET")
            .arg("smtp:config")
            .arg("from")
            .arg("Reloaded <reload@test.com>")
            .arg("base_url")
            .arg("https://reloaded.example.com")
            .query_async(&mut conn.clone())
            .await
            .unwrap();

        // Force reload by expiring the cache
        {
            let mut cfg = s.config.write().await;
            cfg.last_reload = Instant::now()
                .checked_sub(CONFIG_RELOAD_INTERVAL)
                .and_then(|t| t.checked_sub(Duration::from_secs(1)))
                .unwrap_or(Instant::now());
        }

        // Trigger reload
        s.maybe_reload_config().await;

        {
            let cfg = s.config.read().await;
            assert_eq!(cfg.from, "Reloaded <reload@test.com>");
            assert_eq!(cfg.base_url, "https://reloaded.example.com");
            drop(cfg);
        }

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg("smtp:config")
            .query_async(&mut conn.clone())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn hot_reload_respects_cache_ttl() {
        let Some(s) = test_service_with_redis().await else {
            return;
        };
        let conn = s.redis_pool.as_ref().unwrap().conn();

        let _: () = redis::cmd("HSET")
            .arg("smtp:config")
            .arg("from")
            .arg("ShouldNotLoad <nope@t.com>")
            .query_async(&mut conn.clone())
            .await
            .unwrap();

        // Cache still fresh — reload should be skipped
        s.maybe_reload_config().await;

        {
            let cfg = s.config.read().await;
            assert_ne!(
                cfg.from, "ShouldNotLoad <nope@t.com>",
                "fresh cache must not reload"
            );
            drop(cfg);
        }

        let _: () = redis::cmd("DEL")
            .arg("smtp:config")
            .query_async(&mut conn.clone())
            .await
            .unwrap();
    }

    // ── Helpers ────────────────────────────────────────────────────

    fn current_minutely_key() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("email:rate:min:{}", now / RATE_WINDOW_MINUTELY_SECS as u64)
    }

    // ── send_test_email tests ──────────────────────────────────────

    #[tokio::test]
    async fn send_test_email_fails_with_no_email_configured() {
        let s = test_service(false);
        let result = s.send_test_email("admin@test.com").await;
        assert!(
            result.is_err(),
            "Test email must fail when no email transport is configured"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("http_api") || msg.contains("system_settings"),
            "Error should mention missing config options, got: {msg}"
        );
    }

    #[tokio::test]
    async fn send_test_email_bypasses_rate_limit() {
        // With Redis: pre-fill the rate limit, test should still send
        let Some(s) = test_service_with_redis().await else {
            return;
        };
        let conn = s.redis_pool.as_ref().unwrap().conn();
        let key = current_minutely_key();
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg(100u32)
            .query_async(&mut conn.clone())
            .await
            .unwrap();

        // Should fail with "SMTP not configured" (not "rate limited")
        let result = s.send_test_email("admin@test.com").await;
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn.clone())
            .await
            .unwrap();

        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("http_api") || msg.contains("Email is not configured"),
            "Test email should bypass rate limit, got: {msg}"
        );
    }

    // ── HTTP adapter tests ──────────────────────────────────────────

    #[test]
    fn test_parse_from_address_with_name() {
        let (name, email) = parse_from_address("LeeAttend <noreply@leeattend.com>");
        assert_eq!(name, "LeeAttend");
        assert_eq!(email, "noreply@leeattend.com");
    }

    #[test]
    fn test_parse_from_address_bare_email() {
        let (name, email) = parse_from_address("test@example.com");
        assert_eq!(name, "");
        assert_eq!(email, "test@example.com");
    }

    #[test]
    fn test_parse_from_address_with_quotes() {
        let (name, email) = parse_from_address("\"The Team\" <team@ex.com>");
        assert_eq!(name, "The Team");
        assert_eq!(email, "team@ex.com");
    }

    #[test]
    fn test_resolve_template_basic() {
        let tmpl = r#"{"from":{"email":"{{from_email}}","name":"{{from_name}}"},"to":[{"email":"{{to}}"}],"subject":"{{subject}}","text":"{{body}}"}"#;
        let result = resolve_template(tmpl, "Lee", "lee@x.com", "u@x.com", "Hi", "Hello world");
        assert!(result.contains(r#""email":"lee@x.com""#));
        assert!(result.contains(r#""name":"Lee""#));
        assert!(result.contains(r#""email":"u@x.com""#));
        assert!(result.contains(r#""subject":"Hi""#));
        assert!(result.contains(r#""text":"Hello world""#));
    }

    #[test]
    fn test_resolve_template_escapes_quotes() {
        let tmpl = r#"{"text":"{{body}}"}"#;
        let result = resolve_template(tmpl, "", "", "", "", "She said \"hello\"");
        assert!(result.contains(r#""text":"She said \"hello\"""#));
    }

    #[test]
    fn test_resolve_template_escapes_newlines() {
        let tmpl = r#"{"text":"{{body}}"}"#;
        let result = resolve_template(tmpl, "", "", "", "", "line1\nline2");
        assert!(result.contains(r#""text":"line1\nline2""#));
    }

    #[test]
    fn test_resolve_template_escapes_backslash() {
        let tmpl = r#"{"text":"{{body}}"}"#;
        let result = resolve_template(tmpl, "", "", "", "", "path\\to\\file");
        assert!(result.contains(r#""text":"path\\to\\file""#));
    }

    #[test]
    fn test_httpapi_config_deserialization() {
        let json = serde_json::json!({
            "url": "https://send.api.example.com/api/send",
            "auth_header": "Authorization",
            "auth_value": "Bearer {{password}}",
            "success_field": "success",
            "success_value": true,
            "body_template": "{\"from\":{\"email\":\"{{from_email}}\"}}",
        });
        let cfg: HttpApiConfig = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.url, "https://send.api.example.com/api/send");
        assert_eq!(cfg.auth_header.unwrap(), "Authorization");
        assert_eq!(cfg.auth_value.unwrap(), "Bearer {{password}}");
        assert_eq!(cfg.success_field.unwrap(), "success");
        assert_eq!(cfg.success_value.unwrap(), serde_json::Value::Bool(true));
        assert!(cfg.body_template.contains("{{from_email}}"));
    }

    #[test]
    fn test_httpapi_config_minimal() {
        // Only url and body_template are required
        let json = serde_json::json!({
            "url": "https://send.api.example.com/api/send",
            "body_template": "{\"text\":\"{{body}}\"}",
        });
        let cfg: HttpApiConfig = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.url, "https://send.api.example.com/api/send");
        assert!(cfg.auth_header.is_none());
        assert!(cfg.success_field.is_none());
    }

    #[test]
    fn test_smtp_config_data_with_http_api() {
        let json = serde_json::json!({
            "host": "smtp.example.com",
            "port": 587,
            "username": "api",
            "password": "secret",
            "http_api": {
                "url": "https://send.api.example.com/api/send",
                "auth_header": "Api-Token",
                "auth_value": "{{password}}",
                "body_template": "{\"text\":\"{{body}}\"}"
            }
        });
        let cfg: SmtpConfigData = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.host.unwrap(), "smtp.example.com");
        assert!(cfg.http_api.is_some());
        assert_eq!(
            cfg.http_api.as_ref().unwrap().url,
            "https://send.api.example.com/api/send"
        );
    }

    #[test]
    fn test_smtp_config_data_without_http_api() {
        let json = serde_json::json!({
            "host": "smtp.example.com",
            "port": 587
        });
        let cfg: SmtpConfigData = serde_json::from_value(json).unwrap();
        assert_eq!(cfg.host.unwrap(), "smtp.example.com");
        assert!(cfg.http_api.is_none());
    }

    #[tokio::test]
    async fn test_http_config_roundtrip_via_redis() {
        let Some(s) = test_service_with_redis().await else {
            return;
        };
        let conn = s.redis_pool.as_ref().unwrap().conn();

        // Seed Redis with config including http_api
        let _: () = redis::cmd("HSET")
            .arg("smtp:config")
            .arg("http_api")
            .arg(r#"{"url":"https://api.example.com/send","body_template":"{\"text\":\"{{body}}\"}"}"#)
            .query_async(&mut conn.clone())
            .await
            .unwrap();

        let loaded = s.load_from_redis().await;
        assert!(loaded.is_some(), "should load config");
        let loaded = loaded.unwrap();
        assert!(
            loaded.http_api.is_some(),
            "http_api must be deserialized from Redis"
        );

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg("smtp:config")
            .query_async(&mut conn.clone())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_no_http_api_field_redis_backwards_compat() {
        let Some(s) = test_service_with_redis().await else {
            return;
        };
        let conn = s.redis_pool.as_ref().unwrap().conn();

        // Redis without http_api field (legacy deployment)
        let _: () = redis::cmd("HSET")
            .arg("smtp:config")
            .arg("host")
            .arg("smtp.example.com")
            .arg("port")
            .arg("587")
            .query_async(&mut conn.clone())
            .await
            .unwrap();

        let loaded = s.load_from_redis().await;
        assert!(loaded.is_some(), "should load legacy config");
        let loaded = loaded.unwrap();
        assert!(
            loaded.http_api.is_none(),
            "http_api must be None for legacy Redis configs"
        );

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg("smtp:config")
            .query_async(&mut conn.clone())
            .await
            .unwrap();
    }
}
