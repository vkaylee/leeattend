/// Configuration for all database pools, read from environment variables.
#[derive(Debug, Clone)]
pub struct DbPoolConfig {
    /// Primary database URL (read + write). Used for migrations.
    pub primary_url: String,
    /// Transaction Pooler URL (Port 6543)
    pub pooler_tx_url: String,
    /// Session Pooler URL (Port 5452)
    pub pooler_sess_url: String,
    /// Optional read-replica URL. Falls back to `pooler_tx_url` then `primary_url`.
    pub reader_url: Option<String>,

    /// API pool: max connections (default: 20)
    pub api_max_conn: u32,
    /// API pool: acquire timeout in seconds (default: 5)
    pub api_acquire_timeout_secs: u64,

    /// Worker pool: max connections (default: 5)
    pub worker_max_conn: u32,
    /// Worker pool: acquire timeout in seconds (default: 30)
    pub worker_acquire_timeout_secs: u64,

    /// Tenant pool: max connections (default: 5)
    pub tenant_max_conn: u32,
}

impl DbPoolConfig {
    pub fn from_env() -> Result<Self, crate::errors::AppError> {
        let primary_url = std::env::var("DB_PRIMARY_URL")
            .or_else(|_| std::env::var("DB_URL"))
            .or_else(|_| std::env::var("DATABASE_URL"))
            .map_err(|_e| {
                crate::errors::AppError::Internal(
                    "DB_PRIMARY_URL or DATABASE_URL must be set".to_string(),
                )
            })?;
        validate_db_url(&primary_url, "DB_PRIMARY_URL")?;

        let pooler_tx_url = std::env::var("DB_POOLER_TX_URL")
            .or_else(|_| std::env::var("DATABASE_POOLER_TX_URL"))
            .ok()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                crate::errors::AppError::Internal(
                    "DB_POOLER_TX_URL or DATABASE_POOLER_TX_URL must be set (Supavisor mode is enforced)".to_string(),
                )
            })?;
        validate_db_url(&pooler_tx_url, "DB_POOLER_TX_URL")?;

        let pooler_sess_url = std::env::var("DB_POOLER_SESS_URL")
            .or_else(|_| std::env::var("DATABASE_POOLER_SESS_URL"))
            .ok()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                crate::errors::AppError::Internal(
                    "DB_POOLER_SESS_URL or DATABASE_POOLER_SESS_URL must be set (Supavisor mode is enforced)".to_string(),
                )
            })?;
        validate_db_url(&pooler_sess_url, "DB_POOLER_SESS_URL")?;

        let reader_url = std::env::var("DB_READER_URL")
            .or_else(|_| std::env::var("DATABASE_READER_URL"))
            .ok()
            .filter(|s| !s.trim().is_empty());
        if let Some(ref url) = reader_url {
            validate_db_url(url, "DB_READER_URL")?;
        }

        Ok(Self {
            primary_url,
            pooler_tx_url,
            pooler_sess_url,
            reader_url,
            api_max_conn: env_parse("DB_API_MAX_CONN", 20),
            api_acquire_timeout_secs: env_parse("DB_API_ACQUIRE_TIMEOUT_SECS", 5),
            worker_max_conn: env_parse("DB_WORKER_MAX_CONN", 5),
            worker_acquire_timeout_secs: env_parse("DB_WORKER_ACQUIRE_TIMEOUT_SECS", 30),
            tenant_max_conn: env_parse("DB_TENANT_MAX_CONN", 5),
        })
    }

    pub fn resolve_pool_urls(&self) -> (&str, &str, &str) {
        let api_worker_url = &self.pooler_tx_url;
        let reader_url = self.reader_url.as_deref().unwrap_or(api_worker_url);
        let sess_url = &self.pooler_sess_url;

        (api_worker_url, sess_url, reader_url)
    }
}

/// Parse an env var with a fallback default.
fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// Validate database connection string prefix.
fn validate_db_url(url: &str, var_name: &str) -> Result<(), crate::errors::AppError> {
    if !url.starts_with("postgres://") && !url.starts_with("postgresql://") {
        return Err(crate::errors::AppError::Internal(format!(
            "FATAL: {var_name} must start with 'postgres://' or 'postgresql://'"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    fn setup_env(
        db_url: Option<&str>,
        pool_tx: Option<&str>,
        pool_sess: Option<&str>,
        reader: Option<&str>,
    ) {
        let vars = [
            ("DATABASE_URL", db_url),
            ("DB_PRIMARY_URL", db_url),
            ("DB_URL", db_url),
            ("DATABASE_POOLER_TX_URL", pool_tx),
            ("DB_POOLER_TX_URL", pool_tx),
            ("DATABASE_POOLER_SESS_URL", pool_sess),
            ("DB_POOLER_SESS_URL", pool_sess),
            ("DATABASE_READER_URL", reader),
            ("DB_READER_URL", reader),
            ("DB_API_MAX_CONN", None),
            ("DB_API_ACQUIRE_TIMEOUT_SECS", None),
            ("DB_WORKER_MAX_CONN", None),
            ("DB_WORKER_ACQUIRE_TIMEOUT_SECS", None),
            ("DB_TENANT_MAX_CONN", None),
        ];

        for (k, v) in &vars {
            if let Some(val) = v {
                env::set_var(k, val);
            } else {
                env::remove_var(k);
            }
        }
    }

    #[test]
    #[serial]
    fn test_from_env_success() {
        setup_env(
            Some("postgres://user:pass@localhost:5432/db"),
            Some("postgres://pooler:pass@localhost:6543/db"),
            Some("postgres://pooler:pass@localhost:5452/db"),
            None,
        );

        let config = DbPoolConfig::from_env().expect("Should parse config");
        assert_eq!(config.primary_url, "postgres://user:pass@localhost:5432/db");
        assert_eq!(
            config.pooler_tx_url,
            "postgres://pooler:pass@localhost:6543/db"
        );
        assert_eq!(
            config.pooler_sess_url,
            "postgres://pooler:pass@localhost:5452/db"
        );
        assert_eq!(config.reader_url, None);
        assert_eq!(config.api_max_conn, 20);

        let (api_tx, sess, reader) = config.resolve_pool_urls();
        assert_eq!(api_tx, "postgres://pooler:pass@localhost:6543/db");
        assert_eq!(sess, "postgres://pooler:pass@localhost:5452/db");
        assert_eq!(reader, "postgres://pooler:pass@localhost:6543/db"); // Fallback to tx url
    }

    #[test]
    #[serial]
    fn test_from_env_with_reader() {
        setup_env(
            Some("postgres://user:pass@localhost:5432/db"),
            Some("postgres://pooler:pass@localhost:6543/db"),
            Some("postgres://pooler:pass@localhost:5452/db"),
            Some("postgres://reader:pass@localhost:5432/db"),
        );

        let config = DbPoolConfig::from_env().unwrap();
        let (_, _, reader) = config.resolve_pool_urls();
        assert_eq!(reader, "postgres://reader:pass@localhost:5432/db");
    }

    #[test]
    #[serial]
    fn test_from_env_missing_urls() {
        setup_env(None, None, None, None);
        assert!(DbPoolConfig::from_env().is_err());

        setup_env(Some("url"), None, None, None);
        assert!(DbPoolConfig::from_env().is_err());

        setup_env(Some("url"), Some("tx"), None, None);
        assert!(DbPoolConfig::from_env().is_err());
    }

    #[test]
    #[serial]
    fn test_env_parse_defaults_and_overrides() {
        setup_env(
            Some("postgres://user:pass@localhost:5432/db"),
            Some("postgres://pooler:pass@localhost:6543/db"),
            Some("postgres://pooler:pass@localhost:5452/db"),
            None,
        );
        env::set_var("DB_API_MAX_CONN", "42");
        env::set_var("DB_WORKER_ACQUIRE_TIMEOUT_SECS", "99");

        let config = DbPoolConfig::from_env().unwrap();
        assert_eq!(config.api_max_conn, 42);
        assert_eq!(config.api_acquire_timeout_secs, 5); // default
        assert_eq!(config.worker_acquire_timeout_secs, 99);
        assert_eq!(config.tenant_max_conn, 5); // default
    }

    #[test]
    #[serial]
    fn test_from_env_empty_string_urls() {
        setup_env(
            Some("postgres://user:pass@localhost:5432/db"),
            Some("   "), // Empty whitespace pooler tx url
            Some("postgres://pooler:pass@localhost:5452/db"),
            None,
        );
        assert!(DbPoolConfig::from_env().is_err());
    }

    #[test]
    #[serial]
    fn test_env_parse_invalid_values() {
        setup_env(
            Some("postgres://user:pass@localhost:5432/db"),
            Some("postgres://pooler:pass@localhost:6543/db"),
            Some("postgres://pooler:pass@localhost:5452/db"),
            None,
        );
        env::set_var("DB_API_MAX_CONN", "not_a_number");
        env::set_var("DB_WORKER_ACQUIRE_TIMEOUT_SECS", "12.34");

        let config = DbPoolConfig::from_env().unwrap();
        assert_eq!(config.api_max_conn, 20); // fallback to default
        assert_eq!(config.worker_acquire_timeout_secs, 30); // fallback to default
    }

    #[test]
    #[serial]
    fn test_from_env_invalid_url_prefixes() {
        setup_env(
            Some("mysql://user:pass@localhost:5432/db"), // Invalid prefix
            Some("postgres://pooler:pass@localhost:6543/db"),
            Some("postgres://pooler:pass@localhost:5452/db"),
            None,
        );
        let err = DbPoolConfig::from_env();
        assert!(err.is_err());
        if let Err(crate::errors::AppError::Internal(msg)) = err {
            assert!(msg.contains("must start with 'postgres://' or 'postgresql://'"));
        } else {
            panic!("Expected AppError::Internal with prefix validation message");
        }
    }
}
