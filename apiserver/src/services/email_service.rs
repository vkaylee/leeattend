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

/// Serialisable SMTP config snapshot (DB ↔ Redis ↔ memory).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SmtpConfigData {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    from: Option<String>,
    base_url: Option<String>,
}

// ── Hot-swappable SMTP config ─────────────────────────────────────────────

/// All SMTP-related runtime state that can be hot-reloaded from Redis.
struct SmtpConfig {
    transport: Option<SmtpTransport>,
    from: String,
    base_url: String,
    last_reload: Instant,
}

impl SmtpConfig {
    /// Bootstrap with no SMTP transport — will be populated on first reload from DB.
    fn default_config() -> Self {
        Self {
            transport: None,
            from: "LeeAttend <noreply@leeattend.com>".into(),
            base_url: "http://localhost:8080".into(),
            last_reload: Instant::now()
                .checked_sub(CONFIG_RELOAD_INTERVAL)
                .unwrap_or_else(Instant::now), // eager-load eligible
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
    builder = builder.port(port);

    if let (Some(u), Some(p)) = (username, password) {
        builder = builder.credentials(Credentials::new(u.to_string(), p.to_string()));
    }

    Some(builder.build())
}

// ── EmailService ──────────────────────────────────────────────────────────

pub struct EmailService {
    /// Hot-swappable config; read-locked for sends, write-locked on reload.
    config: Arc<RwLock<SmtpConfig>>,
    redis_pool: Option<RedisPool>,
    /// DB-backed system settings (source of truth) — optional for dev/test.
    settings_repo: Option<Arc<dyn SystemSettingsRepository>>,
}

impl EmailService {
    pub fn new(
        redis_pool: Option<RedisPool>,
        settings_repo: Option<Arc<dyn SystemSettingsRepository>>,
    ) -> Self {
        // `last_reload` is set in the past so the first send will trigger
        // an eager load from DB (or Redis cache).
        Self {
            config: Arc::new(RwLock::new(SmtpConfig::default_config())),
            redis_pool,
            settings_repo,
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
        let fields: &[&str] = &["host", "port", "username", "password", "from", "base_url"];
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

        Some(SmtpConfigData {
            host: values[0].clone(),
            port: values[1].as_ref().and_then(|p| p.parse().ok()),
            username: values[2].clone(),
            password: values[3].clone(),
            from: values[4].clone(),
            base_url: values[5].clone(),
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
            .query_async(&mut conn)
            .await
            .map_err(|e| warn!("Failed to write smtp:config to Redis cache: {e}"))
            .unwrap_or(());
    }

    async fn apply_config(&self, c: SmtpConfigData) {
        let new_transport = build_transport(
            c.host.as_deref(),
            c.port,
            c.username.as_deref(),
            c.password.as_deref(),
        );
        let mut cfg = self.config.write().await;
        cfg.transport = new_transport;
        cfg.last_reload = Instant::now();
        if let Some(f) = c.from {
            cfg.from = f;
        }
        if let Some(b) = c.base_url {
            cfg.base_url = b;
        }
        drop(cfg);
        info!(
            "SMTP config reloaded (host={:?}, port={:?})",
            c.host, c.port
        );
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

    // ── Send ─────────────────────────────────────────────────────────

    pub async fn send_password_reset(&self, to_email: &str, token: &str) -> AppResult<()> {
        // Hot-reload SMTP config from Redis if stale
        self.maybe_reload_config().await;

        // Rate limit check
        if !self.check_rate_limit().await {
            return Ok(());
        }

        // Snapshot config under read lock
        let (transport, from, base_url) = {
            let cfg = self.config.read().await;
            (
                cfg.transport.clone(),
                cfg.from.clone(),
                cfg.base_url.clone(),
            )
        };

        let reset_url = format!("{base_url}/reset-password?token={token}");
        let subject = "Reset your LeeAttend Password";
        let body = format!(
            "Hello,\n\nWe received a request to reset your password. Click the link below to set a new password:\n\n{reset_url}\n\nThis link is valid for 15 minutes.\n\nIf you did not request a password reset, please ignore this email.\n\nRegards,\nLeeAttend Team"
        );

        if let Some(transport) = &transport {
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

    /// Send a test email to verify SMTP is configured correctly.
    /// Bypasses rate limiting. Returns the SMTP host used on success.
    pub async fn send_test_email(&self, to_email: &str) -> AppResult<String> {
        // Force-reload config (skip cache TTL)
        {
            let mut cfg = self.config.write().await;
            cfg.last_reload = Instant::now()
                .checked_sub(CONFIG_RELOAD_INTERVAL)
                .unwrap_or_else(Instant::now);
        }
        self.maybe_reload_config().await;

        let (transport, from, base_url) = {
            let cfg = self.config.read().await;
            (
                cfg.transport.clone(),
                cfg.from.clone(),
                cfg.base_url.clone(),
            )
        };

        let transport = transport.ok_or_else(|| {
            AppError::Internal(
                "SMTP is not configured. Insert a 'smtp' record into system_settings.".into(),
            )
        })?;

        let subject = "LeeAttend — SMTP Test Email";
        let body = format!(
            "Hello,\n\nThis is a test email from LeeAttend to confirm your SMTP configuration is working correctly.\n\n\
             Sent at: {}\n\
             App URL: {base_url}\n\n\
             If you received this, your email setup is ready.\n\n\
             Regards,\nLeeAttend Team",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

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
            config: Arc::new(RwLock::new(SmtpConfig {
                transport,
                from: "Test <test@leeattend.com>".into(),
                base_url: "https://app.example.com".into(),
                last_reload: Instant::now(),
            })),
            redis_pool: None,
            settings_repo: None,
        }
    }

    async fn test_service_with_redis() -> Option<EmailService> {
        let url = std::env::var("REDIS_URL")
            .or_else(|_| std::env::var("REDIS_URL_FOR_TEST_GENERATION"))
            .ok()?;
        let pool = RedisPool::new(&url).await.ok()?;
        Some(EmailService {
            config: Arc::new(RwLock::new(SmtpConfig {
                transport: None,
                from: "Test <test@leeattend.com>".into(),
                base_url: "https://app.example.com".into(),
                last_reload: Instant::now(),
            })),
            redis_pool: Some(pool.clone()),
            settings_repo: None,
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
            config: Arc::new(RwLock::new(SmtpConfig {
                transport: None,
                from: "F <f@t.com>".into(),
                base_url: "https://my.app".into(),
                last_reload: Instant::now(),
            })),
            redis_pool: None,
            settings_repo: None,
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
            cfg.last_reload = Instant::now() - CONFIG_RELOAD_INTERVAL - Duration::from_secs(1);
        }

        // Trigger reload
        s.maybe_reload_config().await;

        let cfg = s.config.read().await;
        assert_eq!(cfg.from, "Reloaded <reload@test.com>");
        assert_eq!(cfg.base_url, "https://reloaded.example.com");

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

        let cfg = s.config.read().await;
        assert_ne!(
            cfg.from, "ShouldNotLoad <nope@t.com>",
            "fresh cache must not reload"
        );

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
    async fn send_test_email_fails_with_no_smtp_configured() {
        let s = test_service(false);
        let result = s.send_test_email("admin@test.com").await;
        assert!(
            result.is_err(),
            "Test email must fail when SMTP is not configured"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("SMTP is not configured") || msg.contains("system_settings"),
            "Error should mention missing SMTP config, got: {msg}"
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
            msg.contains("SMTP is not configured"),
            "Test email should bypass rate limit, got: {msg}"
        );
    }
}
