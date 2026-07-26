/// Configuration for all database pools, read from environment variables.
#[derive(Debug, Clone)]
pub struct DbPoolConfig {
    /// Primary database URL (read + write). Used for migrations.
    pub primary_url: String,
    /// Transaction Pooler URL (Port 6543)
    pub pooler_tx_url: String,
    /// Session Pooler URL (Port 5452)
    pub pooler_sess_url: String,
    /// Tenant Transaction Pooler URL
    pub tenant_pooler_tx_url: String,
    /// Tenant Session Pooler URL
    pub tenant_pooler_sess_url: String,
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
    /// Build configuration from the centralized env registry.
    pub fn from_registry(
        registry: &crate::utils::env_registry::EnvRegistry,
    ) -> Result<Self, crate::errors::AppError> {
        let primary_url = registry
            .get("DB_PRIMARY_URL")
            .map(ToString::to_string)
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                crate::errors::AppError::Internal(
                    "DB_PRIMARY_URL or DATABASE_URL must be set".to_string(),
                )
            })?;
        validate_db_url(&primary_url, "DB_PRIMARY_URL")?;

        let pooler_tx_url = registry
            .get("DB_POOLER_TX_URL")
            .map(ToString::to_string)
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                crate::errors::AppError::Internal(
                    "DB_POOLER_TX_URL or DATABASE_POOLER_TX_URL must be set (Supavisor mode is enforced)".to_string(),
                )
            })?;
        validate_db_url(&pooler_tx_url, "DB_POOLER_TX_URL")?;

        let pooler_sess_url = registry
            .get("DB_POOLER_SESS_URL")
            .map(ToString::to_string)
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                crate::errors::AppError::Internal(
                    "DB_POOLER_SESS_URL or DATABASE_POOLER_SESS_URL must be set (Supavisor mode is enforced)".to_string(),
                )
            })?;
        validate_db_url(&pooler_sess_url, "DB_POOLER_SESS_URL")?;

        let tenant_pooler_tx_url = registry
            .get("DB_TENANT_POOLER_TX_URL")
            .map(ToString::to_string)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| pooler_tx_url.clone());
        validate_db_url(&tenant_pooler_tx_url, "DB_TENANT_POOLER_TX_URL")?;

        let tenant_pooler_sess_url = registry
            .get("DB_TENANT_POOLER_SESS_URL")
            .map(ToString::to_string)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| pooler_sess_url.clone());
        validate_db_url(&tenant_pooler_sess_url, "DB_TENANT_POOLER_SESS_URL")?;

        let reader_url = registry
            .get("DB_READER_URL")
            .map(ToString::to_string)
            .filter(|s| !s.trim().is_empty());
        if let Some(ref url) = reader_url {
            validate_db_url(url, "DB_READER_URL")?;
        }

        Ok(Self {
            primary_url,
            pooler_tx_url,
            pooler_sess_url,
            tenant_pooler_tx_url,
            tenant_pooler_sess_url,
            reader_url,
            api_max_conn: registry.get_parsed::<u32>("DB_API_MAX_CONN").unwrap_or(20),
            api_acquire_timeout_secs: registry
                .get_parsed::<u64>("DB_API_ACQUIRE_TIMEOUT_SECS")
                .unwrap_or(5),
            worker_max_conn: 5, // system worker pool — low volume, no env needed
            worker_acquire_timeout_secs: registry
                .get_parsed::<u64>("DB_WORKER_ACQUIRE_TIMEOUT_SECS")
                .unwrap_or(30),
            tenant_max_conn: registry
                .get_parsed::<u32>("DB_TENANT_MAX_CONN")
                .unwrap_or(5),
        })
    }

    pub fn resolve_pool_urls(&self) -> (&str, &str, &str) {
        let api_worker_url = &self.pooler_tx_url;
        let reader_url = self.reader_url.as_deref().unwrap_or(api_worker_url);
        let sess_url = &self.pooler_sess_url;

        (api_worker_url, sess_url, reader_url)
    }
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
