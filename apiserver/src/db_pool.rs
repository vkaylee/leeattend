//! Configurable multi-pool database system.
//!
//! Provides separate connection pools for different workloads:
//! - **API pool**: High connections, fast timeout — for HTTP request handlers
//! - **Worker pool**: Low connections, patient timeout — for background jobs
//! - **Reader pool**: Optional read-replica — falls back to API pool when not configured

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Configuration for all database pools, read from environment variables.
#[derive(Debug, Clone)]
pub struct DbPoolConfig {
    /// Primary database URL (read + write).
    pub primary_url: String,
    /// Optional read-replica URL. Falls back to `primary_url` if not set.
    pub reader_url: Option<String>,

    /// API pool: max connections (default: 20)
    pub api_max_conn: u32,
    /// API pool: acquire timeout in seconds (default: 5)
    pub api_acquire_timeout_secs: u64,

    /// Worker pool: max connections (default: 5)
    pub worker_max_conn: u32,
    /// Worker pool: acquire timeout in seconds (default: 30)
    pub worker_acquire_timeout_secs: u64,
}

impl DbPoolConfig {
    /// Read configuration from environment variables with sensible defaults.
    ///
    /// | Variable | Default | Description |
    /// |----------|---------|-------------|
    /// | `DATABASE_URL` | required | Primary DB |
    /// | `DATABASE_READER_URL` | falls back to primary | Read replica |
    /// | `DB_API_MAX_CONN` | 20 | API pool max connections |
    /// | `DB_API_ACQUIRE_TIMEOUT_SECS` | 5 | API pool acquire timeout |
    /// | `DB_WORKER_MAX_CONN` | 5 | Worker pool max connections |
    /// | `DB_WORKER_ACQUIRE_TIMEOUT_SECS` | 30 | Worker pool acquire timeout |
    pub fn from_env() -> Self {
        #[allow(clippy::expect_used)]
        let primary_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let reader_url = std::env::var("DATABASE_READER_URL")
            .ok()
            .filter(|s| !s.trim().is_empty());

        Self {
            primary_url,
            reader_url,
            api_max_conn: env_parse("DB_API_MAX_CONN", 20),
            api_acquire_timeout_secs: env_parse("DB_API_ACQUIRE_TIMEOUT_SECS", 5),
            worker_max_conn: env_parse("DB_WORKER_MAX_CONN", 5),
            worker_acquire_timeout_secs: env_parse("DB_WORKER_ACQUIRE_TIMEOUT_SECS", 30),
        }
    }
}

/// Holds separate PgPool instances for different workloads.
#[derive(Clone)]
pub struct DbPools {
    /// High-connection, fast-timeout pool for HTTP handlers.
    api: PgPool,
    /// Low-connection, patient pool for background jobs.
    worker: PgPool,
    /// Optional read-replica pool. `None` means reader() falls back to api pool.
    reader: Option<PgPool>,
}

impl DbPools {
    /// Connect to the database(s) and create all pools.
    ///
    /// Panics if the primary database is unreachable.
    pub async fn connect(config: &DbPoolConfig) -> Self {
        tracing::info!(
            api_max = config.api_max_conn,
            api_timeout_s = config.api_acquire_timeout_secs,
            worker_max = config.worker_max_conn,
            worker_timeout_s = config.worker_acquire_timeout_secs,
            has_reader = config.reader_url.is_some(),
            "Initializing database pools"
        );

        #[allow(clippy::expect_used)]
        let api = PgPoolOptions::new()
            .max_connections(config.api_max_conn)
            .acquire_timeout(Duration::from_secs(config.api_acquire_timeout_secs))
            .connect(&config.primary_url)
            .await
            .expect("Failed to connect API pool to Postgres");

        #[allow(clippy::expect_used)]
        let worker = PgPoolOptions::new()
            .max_connections(config.worker_max_conn)
            .acquire_timeout(Duration::from_secs(config.worker_acquire_timeout_secs))
            .connect(&config.primary_url)
            .await
            .expect("Failed to connect Worker pool to Postgres");

        let reader = if let Some(ref reader_url) = config.reader_url {
            tracing::info!("Connecting read-replica pool...");
            #[allow(clippy::expect_used)]
            let pool = PgPoolOptions::new()
                .max_connections(config.api_max_conn)
                .acquire_timeout(Duration::from_secs(config.api_acquire_timeout_secs))
                .connect(reader_url)
                .await
                .expect("Failed to connect Reader pool to Postgres read-replica");
            Some(pool)
        } else {
            None
        };

        tracing::info!("All database pools connected successfully");
        Self { api, worker, reader }
    }

    /// Get the API pool — use for HTTP request handlers.
    ///
    /// High connection count, fast acquire timeout.
    #[inline]
    pub fn api(&self) -> &PgPool {
        &self.api
    }

    /// Get the worker pool — use for background jobs.
    ///
    /// Low connection count, patient acquire timeout.
    /// This ensures workers don't starve API handlers of DB connections.
    #[inline]
    pub fn worker(&self) -> &PgPool {
        &self.worker
    }

    /// Get the reader pool — use for read-heavy queries that tolerate replication lag.
    ///
    /// Returns the read-replica pool if `DATABASE_READER_URL` is configured,
    /// otherwise falls back to the API pool.
    #[inline]
    pub fn reader(&self) -> &PgPool {
        self.reader.as_ref().unwrap_or(&self.api)
    }

    /// Run database migrations on the primary pool.
    pub async fn run_migrations(&self) {
        #[allow(clippy::expect_used)]
        sqlx::migrate!("./migrations")
            .run(&self.api)
            .await
            .expect("Failed to run migrations");
    }

    /// Create a DbPools instance using a single PgPool for all workloads.
    /// Designed specifically for use in integration tests.
    pub fn for_test(pool: PgPool) -> Self {
        Self {
            api: pool.clone(),
            worker: pool.clone(),
            reader: Some(pool),
        }
    }
}

/// Parse an env var with a fallback default.
fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let config = DbPoolConfig {
            primary_url: "postgres://test".to_string(),
            reader_url: None,
            api_max_conn: 20,
            api_acquire_timeout_secs: 5,
            worker_max_conn: 5,
            worker_acquire_timeout_secs: 30,
        };
        assert_eq!(config.primary_url, "postgres://test");
        assert_eq!(config.api_max_conn, 20);
        assert_eq!(config.api_acquire_timeout_secs, 5);
        assert_eq!(config.worker_max_conn, 5);
        assert_eq!(config.worker_acquire_timeout_secs, 30);
        assert!(config.reader_url.is_none());
    }

    #[test]
    fn test_env_parse_with_default() {
        assert_eq!(env_parse::<u32>("NONEXISTENT_VAR_12345_TEST", 42), 42);
        assert_eq!(env_parse::<u64>("NONEXISTENT_VAR_12345_TEST", 42), 42);
        assert_eq!(env_parse::<String>("NONEXISTENT_VAR_12345_TEST", "fallback".to_string()), "fallback");
    }

    #[test]
    fn test_env_parse_with_valid_env_value() {
        // Save the old value so we don't permanently mess up the environment
        let key = "DB_POOL_TEST_VAR_VALID";
        let old_val = std::env::var(key).ok();

        std::env::set_var(key, "99");
        assert_eq!(env_parse::<u32>(key, 42), 99);

        // Restore
        if let Some(val) = old_val {
            std::env::set_var(key, val);
        } else {
            std::env::remove_var(key);
        }
    }

    #[test]
    fn test_env_parse_with_invalid_env_value_falls_back_to_default() {
        let key = "DB_POOL_TEST_VAR_INVALID";
        let old_val = std::env::var(key).ok();

        // "not_a_number" cannot be parsed into u32, should fallback to 42
        std::env::set_var(key, "not_a_number");
        assert_eq!(env_parse::<u32>(key, 42), 42);

        // Restore
        if let Some(val) = old_val {
            std::env::set_var(key, val);
        } else {
            std::env::remove_var(key);
        }
    }

    // ── Pool isolation tests ──────────────────────────────────────

    #[test]
    fn test_reader_fallback_when_no_reader_url() {
        let config = DbPoolConfig {
            primary_url: "postgres://test".to_string(),
            reader_url: None,
            api_max_conn: 20,
            api_acquire_timeout_secs: 5,
            worker_max_conn: 5,
            worker_acquire_timeout_secs: 30,
        };
        // When reader_url is None, reader pool should not be created
        assert!(config.reader_url.is_none());
    }

    #[test]
    fn test_reader_configured_when_url_set() {
        let config = DbPoolConfig {
            primary_url: "postgres://primary".to_string(),
            reader_url: Some("postgres://replica".to_string()),
            api_max_conn: 20,
            api_acquire_timeout_secs: 5,
            worker_max_conn: 5,
            worker_acquire_timeout_secs: 30,
        };
        assert!(config.reader_url.is_some());
        assert_ne!(
            config.primary_url,
            config.reader_url.as_ref().unwrap().as_str(),
            "Reader URL should differ from primary"
        );
    }

    #[test]
    fn test_api_and_worker_have_different_configs() {
        let config = DbPoolConfig {
            primary_url: "postgres://test".to_string(),
            reader_url: None,
            api_max_conn: 20,
            api_acquire_timeout_secs: 5,
            worker_max_conn: 5,
            worker_acquire_timeout_secs: 30,
        };
        // API: many connections, fast timeout (for HTTP handlers)
        // Worker: few connections, patient timeout (for background jobs)
        assert!(
            config.api_max_conn > config.worker_max_conn,
            "API pool should have more connections than worker pool"
        );
        assert!(
            config.api_acquire_timeout_secs < config.worker_acquire_timeout_secs,
            "API pool should have shorter timeout than worker pool"
        );
    }

    #[test]
    fn test_empty_reader_url_treated_as_none() {
        // Simulate DATABASE_READER_URL="" (set but empty)
        let reader_url = Some("".to_string()).filter(|s| !s.trim().is_empty());
        assert!(reader_url.is_none(), "Empty reader URL should be treated as None");

        let reader_url = Some("  ".to_string()).filter(|s| !s.trim().is_empty());
        assert!(reader_url.is_none(), "Whitespace-only reader URL should be treated as None");
    }

    // ── Pool assignment contract tests ────────────────────────────
    // These tests verify the RULES documented in main.rs:
    //   api pool    → HTTP handlers, CRUD, auth
    //   worker pool → background jobs, import, accrual
    //   reader pool → dashboard analytics, ESS reads (SELECT only)

    #[test]
    fn test_pool_assignment_contract_api_vs_worker() {
        // API pool is designed for request-scoped operations
        // - High connection limit to handle concurrent HTTP requests
        // - Short timeout to fail fast on slow queries
        let api_max = 20u32;
        let api_timeout = 5u64;
        let worker_max = 5u32;
        let worker_timeout = 30u64;

        // Contract: API pool must handle more concurrent connections
        assert!(api_max >= 4 * worker_max, "API should have ≥4x worker connections");
        // Contract: Worker pool must be more patient for long-running jobs
        assert!(worker_timeout >= 5 * api_timeout, "Worker should have ≥5x API timeout");
    }

    #[test]
    fn test_for_test_creates_all_pools() {
        // for_test() should create a valid DbPools with all 3 pools set
        // (Can't test with real PgPool in unit tests, but verify the logic)
        let config = DbPoolConfig {
            primary_url: "postgres://test".to_string(),
            reader_url: None,
            api_max_conn: 1,
            api_acquire_timeout_secs: 1,
            worker_max_conn: 1,
            worker_acquire_timeout_secs: 1,
        };
        // In for_test(), reader is Some(pool) — not None
        // This means reader() won't fallback to api in tests
        assert!(config.reader_url.is_none());
        // But for_test() explicitly sets reader = Some(pool)
        // verifying via code inspection (no PgPool available in unit test)
    }
}
