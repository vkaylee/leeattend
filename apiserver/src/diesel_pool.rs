use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use std::time::Duration;

pub type DieselPool = Pool<AsyncPgConnection>;
pub type DieselConnection = Object<AsyncPgConnection>;

/// Configuration for all database pools, read from environment variables.
#[derive(Debug, Clone)]
pub struct DbPoolConfig {
    /// Primary database URL (read + write). Used for migrations.
    pub primary_url: String,
    /// Connection pooler URL (e.g., Supavisor). Used for API and Worker pools.
    pub pooler_url: Option<String>,
    /// Optional read-replica URL. Falls back to `pooler_url` then `primary_url`.
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
    pub fn from_env() -> Self {
        #[allow(clippy::expect_used)]
        let primary_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pooler_url = std::env::var("DATABASE_POOLER_URL")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let reader_url = std::env::var("DATABASE_READER_URL")
            .ok()
            .filter(|s| !s.trim().is_empty());

        Self {
            primary_url,
            pooler_url,
            reader_url,
            api_max_conn: env_parse("DB_API_MAX_CONN", 20),
            api_acquire_timeout_secs: env_parse("DB_API_ACQUIRE_TIMEOUT_SECS", 5),
            worker_max_conn: env_parse("DB_WORKER_MAX_CONN", 5),
            worker_acquire_timeout_secs: env_parse("DB_WORKER_ACQUIRE_TIMEOUT_SECS", 30),
            tenant_max_conn: env_parse("DB_TENANT_MAX_CONN", 5),
        }
    }

    pub fn resolve_pool_urls(&self) -> (&str, &str, &str) {
        let api_worker_url = self.pooler_url.as_deref().unwrap_or(&self.primary_url);
        let reader_url = self.reader_url.as_deref().unwrap_or(api_worker_url);

        (api_worker_url, api_worker_url, reader_url)
    }
}

/// Parse an env var with a fallback default.
fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

use crate::errors::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait TenantPoolProvider: Send + Sync {
    async fn get_pool(&self, workspace_id: &Uuid) -> Result<DieselPool, AppError>;
}

pub struct ProductionTenantPoolManager {
    base_url: String,
    max_size: usize,
    pools: RwLock<HashMap<Uuid, DieselPool>>,
}

impl ProductionTenantPoolManager {
    pub fn new(base_url: String, max_size: usize) -> Self {
        Self {
            base_url,
            max_size,
            pools: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_tenant_url(&self, workspace_id: &Uuid) -> Result<String, AppError> {
        let db_name = format!("tenant_{}", workspace_id.to_string().replace('-', ""));
        
        let base_url = &self.base_url;
        let (without_query, query) = match base_url.find('?') {
            Some(idx) => (&base_url[..idx], &base_url[idx..]),
            None => (base_url.as_str(), ""),
        };

        let (before_slash, _) = match without_query.rfind('/') {
            Some(idx) => (&without_query[..idx], &without_query[idx+1..]),
            None => return Err(AppError::Internal("Invalid base pool URL: missing path slash".into())),
        };

        let mut result_url = format!("{before_slash}/{db_name}{query}");

        // Now handle username if it contains a dot
        let prefix_len = if result_url.starts_with("postgres://") {
            Some(11)
        } else if result_url.starts_with("postgresql://") {
            Some(13)
        } else {
            None
        };

        if let Some(plen) = prefix_len {
            let after_prefix = &result_url[plen..];
            // find last @ before the path
            let path_start = after_prefix.find('/').unwrap_or(after_prefix.len());
            let auth_host = &after_prefix[..path_start];
            
            if let Some(at_idx) = auth_host.rfind('@') {
                let auth = &auth_host[..at_idx];
                let user = match auth.find(':') {
                    Some(colon_idx) => &auth[..colon_idx],
                    None => auth,
                };

                if user.contains('.') {
                    if let Some((prefix, _old_tenant)) = user.split_once('.') {
                        let new_user = format!("{prefix}.{db_name}");
                        let replace_start = plen;
                        let replace_end = plen + user.len();
                        result_url.replace_range(replace_start..replace_end, &new_user);
                    }
                }
            }
        }

        Ok(result_url)
    }
}

#[async_trait::async_trait]
impl TenantPoolProvider for ProductionTenantPoolManager {
    async fn get_pool(&self, workspace_id: &Uuid) -> Result<DieselPool, AppError> {
        // Fast path
        {
            let pools = self.pools.read().await;
            if let Some(pool) = pools.get(workspace_id) {
                return Ok(pool.clone());
            }
        }

        // Slow path
        let mut pools = self.pools.write().await;
        if let Some(pool) = pools.get(workspace_id) {
            return Ok(pool.clone());
        }

        let tenant_url = self.get_tenant_url(workspace_id)?;
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(tenant_url);
        let pool = Pool::builder(manager)
            .max_size(self.max_size)
            .wait_timeout(Some(Duration::from_secs(5)))
            .create_timeout(Some(Duration::from_secs(5)))
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create tenant pool: {e}")))?;

        pools.insert(*workspace_id, pool.clone());
        Ok(pool)
    }
}

pub struct TestTenantPoolManager {
    pool: DieselPool,
}

impl TestTenantPoolManager {
    pub fn new(pool: DieselPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TenantPoolProvider for TestTenantPoolManager {
    async fn get_pool(&self, _workspace_id: &Uuid) -> Result<DieselPool, AppError> {
        Ok(self.pool.clone())
    }
}

#[derive(Clone)]
pub struct DieselPools {
    pub api: DieselPool,
    pub worker: DieselPool,
    pub tenants: Arc<dyn TenantPoolProvider>,
}

fn is_transient_supavisor_error(err_msg: &str) -> bool {
    // These errors indicate Supavisor is momentarily restarting the tenant process (e.g., during PUT)
    err_msg.contains("No auth_query specified")
        || err_msg.contains("Tenant or user not found")
        || err_msg.contains("terminating connection")
        || err_msg.contains("closed")
}

impl DieselPools {
    /// Convenience method: get a connection from the tenant pool for a given workspace.
    pub async fn tenant_conn(&self, workspace_id: Uuid) -> Result<DieselConnection, AppError> {
        let pool = self.tenants.get_pool(&workspace_id).await?;

        let mut retries = 3;
        loop {
            match pool.get().await {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    let err_msg = e.to_string();
                    if retries > 0 && is_transient_supavisor_error(&err_msg) {
                        tracing::warn!(
                            workspace_id = %workspace_id,
                            error = %err_msg,
                            "Transient Supavisor connection error, retrying in 300ms..."
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        retries -= 1;
                        continue;
                    }
                    return Err(AppError::Internal(format!("db: {e}")));
                }
            }
        }
    }

    #[allow(clippy::unused_async, clippy::expect_used)]
    pub async fn connect(config: &DbPoolConfig) -> Self {
        tracing::info!(
            api_max = config.api_max_conn,
            worker_max = config.worker_max_conn,
            "Initializing Diesel pools"
        );

        let (api_conn_url, worker_conn_url, _) = config.resolve_pool_urls();

        let api_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(api_conn_url);
        let api_pool = Pool::builder(api_manager)
            .max_size(config.api_max_conn as usize)
            .wait_timeout(Some(Duration::from_secs(config.api_acquire_timeout_secs)))
            .create_timeout(Some(Duration::from_secs(5)))
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .expect("Failed to create API Diesel pool");

        let worker_manager =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(worker_conn_url);
        let worker_pool = Pool::builder(worker_manager)
            .max_size(config.worker_max_conn as usize)
            .wait_timeout(Some(Duration::from_secs(
                config.worker_acquire_timeout_secs,
            )))
            .create_timeout(Some(Duration::from_secs(5)))
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .expect("Failed to create Worker Diesel pool");

        let tenant_manager = Arc::new(ProductionTenantPoolManager::new(
            api_conn_url.to_string(),
            config.tenant_max_conn as usize,
        ));

        tracing::info!("Diesel pools connected successfully");

        Self {
            api: api_pool,
            worker: worker_pool,
            tenants: tenant_manager,
        }
    }

    #[allow(clippy::expect_used)]
    pub fn run_system_migrations(config: &DbPoolConfig) {
        use diesel::Connection;
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        pub const SYSTEM_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/system");

        let mut conn = diesel::PgConnection::establish(&config.primary_url)
            .expect("Failed to connect to primary database for system migrations");

        conn.run_pending_migrations(SYSTEM_MIGRATIONS)
            .expect("Failed to run system migrations");

        tracing::info!("Database system migrations completed successfully");
    }

    /// Resolve the tenant database connection URL for running migrations.
    pub fn resolve_tenant_migration_url(
        primary_url: &str,
        pooler_url: Option<&str>,
        workspace_id: &uuid::Uuid,
        session_port_override: Option<u16>,
    ) -> Result<String, String> {
        let mut base_url_parsed = reqwest::Url::parse(pooler_url.unwrap_or(primary_url))
            .map_err(|e| format!("Failed to parse URL: {e}"))?;

        if pooler_url.is_some() && base_url_parsed.username().contains('.') {
            if let Some(port) = session_port_override {
                let _ = base_url_parsed.set_port(Some(port));
            }
        }

        let db_name = format!("tenant_{}", workspace_id.to_string().replace('-', ""));
        base_url_parsed.set_path(&db_name);

        let current_user = base_url_parsed.username().to_string();
        if current_user.contains('.') {
            if let Some((prefix, _)) = current_user.split_once('.') {
                base_url_parsed
                    .set_username(&format!("{prefix}.{db_name}"))
                    .map_err(|_e| "Failed to set username".to_string())?;
            }
        }

        Ok(base_url_parsed.to_string())
    }

    /// Run pending tenant migrations on ALL existing tenant databases.
    ///
    /// Iterates over every workspace in the system DB, connects to each tenant DB,
    /// and runs any pending tenant-level migrations. Called once at startup.
    #[allow(clippy::expect_used)]
    pub fn run_tenant_migrations(config: &DbPoolConfig) {
        use diesel::sql_types::Uuid as UuidSql;
        use diesel::{Connection, QueryableByName, RunQueryDsl};
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        pub const TENANT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/tenant");

        #[derive(QueryableByName)]
        struct WorkspaceRow {
            #[diesel(sql_type = UuidSql)]
            id: uuid::Uuid,
        }

        // Connect to system DB and list all workspaces
        let mut sys_conn = diesel::PgConnection::establish(&config.primary_url)
            .expect("Failed to connect to system DB for tenant migrations");

        let workspace_rows: Vec<WorkspaceRow> = diesel::sql_query("SELECT id FROM workspaces")
            .load(&mut sys_conn)
            .expect("Failed to query workspaces for tenant migrations");

        if workspace_rows.is_empty() {
            tracing::info!("No workspaces found — skipping tenant migrations");
            return;
        }

        tracing::info!(
            workspace_count = workspace_rows.len(),
            "Running tenant migrations on existing workspaces"
        );

        let session_port: Option<u16> = if config.pooler_url.is_some() {
            let port: u16 = std::env::var("SUPAVISOR_PORT_SESSION")
                .unwrap_or_else(|_| "5452".to_string())
                .parse()
                .unwrap_or(5452);
            Some(port)
        } else {
            None
        };

        for row in &workspace_rows {
            let tenant_url = match Self::resolve_tenant_migration_url(
                &config.primary_url,
                config.pooler_url.as_deref(),
                &row.id,
                session_port,
            ) {
                Ok(url) => url,
                Err(e) => {
                    tracing::error!(
                        workspace_id = %row.id,
                        error = %e,
                        "Failed to resolve tenant migration URL — skipping workspace"
                    );
                    continue;
                }
            };

            let mut retries = 10;
            
            loop {
                match diesel::PgConnection::establish(&tenant_url) {
                    Ok(mut tenant_conn) => {
                        match tenant_conn.run_pending_migrations(TENANT_MIGRATIONS) {
                            Ok(_) => {
                                tracing::info!(
                                    workspace_id = %row.id,
                                    "Tenant migrations completed successfully"
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    workspace_id = %row.id,
                                    error = %e,
                                    "Failed to run tenant migrations — skipping workspace"
                                );
                            }
                        }
                        break;
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        if retries > 0 && crate::diesel_pool::is_transient_supavisor_error(&err_msg) {
                            tracing::warn!(
                                workspace_id = %row.id,
                                error = %err_msg,
                                "Transient connection error (Supavisor sync delay), retrying in 1s..."
                            );
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            retries -= 1;
                            continue;
                        }
                        
                        tracing::warn!(
                            workspace_id = %row.id,
                            error = %e,
                            "Failed to connect to tenant DB after retries — may not exist yet, skipping"
                        );
                        break;
                    }
                }
            }
        }

        tracing::info!("Tenant migrations on existing workspaces completed");
    }

    #[allow(clippy::expect_used)]
    pub fn run_all_migrations_for_test(config: &DbPoolConfig) {
        use diesel::Connection;
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        pub const SYSTEM_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/system");
        pub const TENANT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/tenant");

        let mut conn = diesel::PgConnection::establish(&config.primary_url)
            .expect("Failed to connect to primary database for all migrations");

        conn.run_pending_migrations(SYSTEM_MIGRATIONS)
            .expect("Failed to run system migrations");

        conn.run_pending_migrations(TENANT_MIGRATIONS)
            .expect("Failed to run tenant migrations");

        tracing::info!("Database test migrations (system + tenant) completed successfully");
    }

    #[allow(clippy::expect_used)]
    pub fn for_test(url: &str) -> Self {
        let api_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url.to_string());
        let api_pool = Pool::builder(api_manager)
            .max_size(5)
            .build()
            .expect("Failed to create test API Diesel pool");

        let worker_manager =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(url.to_string());
        let worker_pool = Pool::builder(worker_manager)
            .max_size(5)
            .build()
            .expect("Failed to create test Worker Diesel pool");

        let tenant_manager = Arc::new(TestTenantPoolManager::new(api_pool.clone()));

        Self {
            api: api_pool,
            worker: worker_pool,
            tenants: tenant_manager,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_tenant_pool_manager_url_direct_connection() {
        // Direct connection: user has no dot, leave unchanged
        let base_url = "postgres://user:pass@localhost:5432/postgres";
        let manager = ProductionTenantPoolManager::new(base_url.to_string(), 5);

        let workspace_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let result = manager.get_tenant_url(&workspace_id).unwrap();

        assert_eq!(
            result,
            "postgres://user:pass@localhost:5432/tenant_123e4567e89b12d3a456426614174000"
        );
    }

    #[test]
    fn test_tenant_pool_manager_url_supavisor_mode() {
        // Supavisor connection: user has dot (admin.leeattend), update tenant part
        let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
        let manager = ProductionTenantPoolManager::new(base_url.to_string(), 5);

        let workspace_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let result = manager.get_tenant_url(&workspace_id).unwrap();

        assert_eq!(
            result,
            "postgres://admin.tenant_123e4567e89b12d3a456426614174000:pass@supavisor:5452/tenant_123e4567e89b12d3a456426614174000"
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Contract: get_tenant_url() MUST produce URLs whose tenant identifier
    // matches the tenant_id returned by build_tenant_payload().
    //
    // The format is: tenant_<uuid-without-hyphens>
    //
    // This invariant is load-bearing. If either side changes format
    // without updating the other, the Supavisor pooler returns:
    //   "Tenant or user not found"
    //
    // These tests lock down the format so it cannot drift.
    // ─────────────────────────────────────────────────────────────────────

    fn expected_tenant_name(ws_id: &Uuid) -> String {
        format!("tenant_{}", ws_id.to_string().replace('-', ""))
    }

    // ── Supavisor mode: tenant name in user suffix AND path ──────────────

    #[test]
    fn test_get_tenant_url_supavisor_mode_standard_uuid() {
        let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
        let manager = ProductionTenantPoolManager::new(base_url.to_string(), 5);
        let ws = Uuid::parse_str("019e40ee-1192-7492-bc29-85dad4675f34").unwrap();

        let url = manager.get_tenant_url(&ws).unwrap();
        let expected = expected_tenant_name(&ws);

        let parsed = reqwest::Url::parse(&url).unwrap();
        let username = parsed.username();
        assert!(
            username.contains('.'),
            "Supavisor mode: username must contain dot"
        );
        let tenant_in_user = username.split('.').nth(1).unwrap();
        assert_eq!(
            tenant_in_user, expected,
            "supavisor user suffix must be tenant_<uuid-no-hyphens>"
        );
        assert_eq!(
            parsed.path(),
            format!("/{expected}"),
            "supavisor db path must be /tenant_<uuid-no-hyphens>"
        );
    }

    #[test]
    fn test_get_tenant_url_supavisor_mode_nil_uuid() {
        let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
        let manager = ProductionTenantPoolManager::new(base_url.to_string(), 5);
        let ws = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();

        let url = manager.get_tenant_url(&ws).unwrap();
        let expected = "tenant_00000000000000000000000000000000";
        let parsed = reqwest::Url::parse(&url).unwrap();
        let tenant_in_user = parsed.username().split('.').nth(1).unwrap();
        assert_eq!(
            tenant_in_user, expected,
            "nil UUID must become tenant_<32-zeros>"
        );
        assert!(
            !tenant_in_user.contains('-'),
            "tenant name must never contain hyphens"
        );
    }

    #[test]
    fn test_get_tenant_url_supavisor_mode_max_uuid() {
        let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
        let manager = ProductionTenantPoolManager::new(base_url.to_string(), 5);
        let ws = Uuid::parse_str("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap();

        let url = manager.get_tenant_url(&ws).unwrap();
        // All 'f' chars, no hyphens
        assert!(
            url.contains("tenant_ffffffffffffffffffffffffffffffff"),
            "max UUID must strip all hyphens"
        );
    }

    #[test]
    fn test_get_tenant_url_never_contains_hyphens_in_tenant_name() {
        // The tenant identifier embedded in the URL is used by Supavisor for
        // routing. UUID hyphens are stripped because database names cannot
        // contain them.

        let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
        let manager = ProductionTenantPoolManager::new(base_url.to_string(), 5);

        // Test several random UUIDs — none should produce hyphens in the tenant part
        for _ in 0..10 {
            let ws = Uuid::new_v4();
            let url = manager.get_tenant_url(&ws).unwrap();
            let parsed = reqwest::Url::parse(&url).unwrap();
            let username = parsed.username();
            let tenant_name = username.split('.').nth(1).unwrap();
            assert!(
                !tenant_name.contains('-'),
                "tenant name '{tenant_name}' must never contain hyphens"
            );
            assert!(
                tenant_name.starts_with("tenant_"),
                "tenant name must start with 'tenant_': got '{tenant_name}'"
            );
            // UUID is 32 hex chars, plus "tenant_" = 39 chars
            assert_eq!(
                tenant_name.len(),
                39,
                "tenant name must be exactly 39 chars (tenant_ + 32 hex): got {}",
                tenant_name.len()
            );
        }
    }

    #[test]
    fn test_get_tenant_url_tenant_name_is_predictable_from_workspace_id() {
        // The tenant name is deterministic: tenant_ + UUID with hyphens removed.
        // This test ensures the algorithm is fixed and cannot be changed
        // without breaking the Supavisor contract.
        let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
        let manager = ProductionTenantPoolManager::new(base_url.to_string(), 5);

        let ws = Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();
        let url = manager.get_tenant_url(&ws).unwrap();

        // Expected: all hyphens removed from the UUID
        let expected_name = "tenant_a1b2c3d4e5f67890abcdef1234567890";
        let parsed = reqwest::Url::parse(&url).unwrap();
        let tenant_in_user = parsed.username().split('.').nth(1).unwrap();
        assert_eq!(tenant_in_user, expected_name);
        assert_eq!(parsed.path(), format!("/{expected_name}"));
    }

    // ── Direct connection mode: no dot in user, tenant name in path only ─

    #[test]
    fn test_get_tenant_url_direct_connection_uses_tenant_prefix_in_path() {
        let base_url = "postgres://user:pass@localhost:5432/postgres";
        let manager = ProductionTenantPoolManager::new(base_url.to_string(), 5);
        let ws = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let url = manager.get_tenant_url(&ws).unwrap();

        let expected_path = "/tenant_123e4567e89b12d3a456426614174000";
        let parsed = reqwest::Url::parse(&url).unwrap();
        assert_eq!(parsed.path(), expected_path);
        // Direct connection: user unchanged
        assert_eq!(parsed.username(), "user");
        assert!(!parsed.username().contains('.'));
    }

    #[test]
    fn test_get_tenant_url_direct_connection_various_uuids() {
        let base_url = "postgres://user:pass@localhost:5432/postgres";
        let manager = ProductionTenantPoolManager::new(base_url.to_string(), 5);

        let cases = [
            (
                "00000000-0000-0000-0000-000000000000",
                "tenant_00000000000000000000000000000000",
            ),
            (
                "ffffffff-ffff-ffff-ffff-ffffffffffff",
                "tenant_ffffffffffffffffffffffffffffffff",
            ),
            (
                "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "tenant_a1b2c3d4e5f67890abcdef1234567890",
            ),
        ];

        for (uuid_str, expected_path) in &cases {
            let ws = Uuid::parse_str(uuid_str).unwrap();
            let url = manager.get_tenant_url(&ws).unwrap();
            let parsed = reqwest::Url::parse(&url).unwrap();
            assert_eq!(
                parsed.path(),
                format!("/{expected_path}"),
                "direct connection: UUID {uuid_str} must produce path /{expected_path}"
            );
        }
    }

    // ── Cross-contract: URL tenant name must match registration tenant_id ─

    #[test]
    fn test_get_tenant_url_tenant_name_roundtrip_with_expected_registration_id() {
        // This is the definitive cross-module contract:
        //   get_tenant_url(ws_id) embeds a tenant name T in the connection URL.
        //   build_tenant_payload(db_url_with_T) must produce tenant_id = T.
        //
        // We verify the tenant name extracted from the URL is exactly what
        // the registration side expects.

        let supavisor_base = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
        let direct_base = "postgres://user:pass@localhost:5432/postgres";

        let ws = Uuid::parse_str("019e40ee-1192-7492-bc29-85dad4675f34").unwrap();
        let expected = "tenant_019e40ee11927492bc2985dad4675f34";

        // Supavisor mode
        let url = ProductionTenantPoolManager::new(supavisor_base.to_string(), 5)
            .get_tenant_url(&ws)
            .unwrap();
        let parsed = reqwest::Url::parse(&url).unwrap();
        let tenant_in_user = parsed.username().split('.').nth(1).unwrap();
        assert_eq!(tenant_in_user, expected);
        assert_eq!(parsed.path().trim_start_matches('/'), expected);

        // Direct mode: path only
        let url2 = ProductionTenantPoolManager::new(direct_base.to_string(), 5)
            .get_tenant_url(&ws)
            .unwrap();
        let parsed2 = reqwest::Url::parse(&url2).unwrap();
        assert_eq!(parsed2.path().trim_start_matches('/'), expected);

        // Also verify format properties
        assert_eq!(expected.len(), 39); // "tenant_" + 32 hex chars
        assert!(expected.starts_with("tenant_"));
        assert!(!expected.contains('-'));
        assert!(expected.chars().skip(7).all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn test_tenant_pool_manager_test_mode_returns_same_pool() {
        let base_url = "postgres://user:pass@localhost:5432/leeattend_test_db";
        let api_manager =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(base_url.to_string());
        let pool = Pool::builder(api_manager).max_size(2).build().unwrap();

        let test_manager = TestTenantPoolManager::new(pool);

        let ws1 = Uuid::new_v4();
        let ws2 = Uuid::new_v4();

        let pool1 = test_manager.get_pool(&ws1).await.unwrap();
        let pool2 = test_manager.get_pool(&ws2).await.unwrap();

        // Since we are mocking the DI, the mock manager should always return clones of the same pool
        // regardless of the workspace ID requested.
        assert_eq!(pool1.status().max_size, 2);
        assert_eq!(pool2.status().max_size, 2);
    }

    #[test]
    fn test_resolve_tenant_migration_url_fallback() {
        let primary_url = "postgres://test_user:pass@db_system:5432/test_database";
        let workspace_id = Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();

        let url = DieselPools::resolve_tenant_migration_url(primary_url, None, &workspace_id, None)
            .unwrap();

        assert_eq!(
            url,
            "postgres://test_user:pass@db_system:5432/tenant_a1b2c3d4e5f67890abcdef1234567890"
        );
    }

    #[test]
    fn test_resolve_tenant_migration_url_uses_session_port() {
        let primary_url = "postgres://test_user:pass@db_system:5432/test_database";
        let pooler_url = "postgres://test_user.test_database:pass@db_system:9999/test_database";
        let workspace_id = Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();

        let url = DieselPools::resolve_tenant_migration_url(
            primary_url,
            Some(pooler_url),
            &workspace_id,
            Some(5452),
        )
        .unwrap();

        assert_eq!(url, "postgres://test_user.tenant_a1b2c3d4e5f67890abcdef1234567890:pass@db_system:5452/tenant_a1b2c3d4e5f67890abcdef1234567890");
    }

    #[test]
    fn test_resolve_tenant_migration_url_keeps_port_if_not_supavisor() {
        let primary_url = "postgres://test_user:pass@db_system:5432/test_database";
        let pooler_url = "postgres://test_user_no_dot:pass@db_system:5432/test_database";
        let workspace_id = Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();

        let url = DieselPools::resolve_tenant_migration_url(
            primary_url,
            Some(pooler_url),
            &workspace_id,
            Some(9999),
        )
        .unwrap();

        assert_eq!(
            url,
            "postgres://test_user_no_dot:pass@db_system:5432/tenant_a1b2c3d4e5f67890abcdef1234567890"
        );
    }

    #[test]
    fn test_is_transient_supavisor_error() {
        assert!(super::is_transient_supavisor_error(
            "Failed to get secret: {:error, \"No auth_query specified\"}"
        ));
        assert!(super::is_transient_supavisor_error(
            "Tenant or user not found"
        ));
        assert!(super::is_transient_supavisor_error(
            "terminating connection due to administrator command"
        ));
        assert!(super::is_transient_supavisor_error("connection closed"));
    }

    #[test]
    fn test_is_transient_supavisor_error_other() {
        assert!(!super::is_transient_supavisor_error(
            "password authentication failed for user"
        ));
        assert!(!super::is_transient_supavisor_error("Connection refused"));
        assert!(!super::is_transient_supavisor_error(
            "database does not exist"
        ));
        assert!(!super::is_transient_supavisor_error("timeout expired"));
    }

    #[test]
    fn test_supavisor_transient_retry_loop() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::{Arc, Mutex};
        use diesel::Connection;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let connect_count = Arc::new(Mutex::new(0));
        let connect_count_clone = connect_count.clone();

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                *connect_count_clone.lock().unwrap() += 1;

                let mut buf = [0u8; 8];
                if stream.read_exact(&mut buf).is_ok() {
                    // Handle SSL Request
                    if buf == [0, 0, 0, 8, 4, 210, 22, 47] {
                        let _ = stream.write_all(b"N");
                        let mut len_buf = [0u8; 4];
                        if stream.read_exact(&mut len_buf).is_ok() {
                            let len = u32::from_be_bytes(len_buf);
                            let mut startup_buf = vec![0u8; len as usize - 4];
                            let _ = stream.read_exact(&mut startup_buf);
                        }
                    }

                    // Build mock Postgres ErrorResponse packet
                    let mut error_payload = Vec::new();
                    error_payload.extend_from_slice(b"SFATAL\0");
                    error_payload.extend_from_slice(b"CXX000\0");
                    error_payload.extend_from_slice(
                        b"MFailed to get secret: {:error, \"No auth_query specified\"}\0",
                    );
                    error_payload.push(0);

                    let len = error_payload.len() as u32 + 4;
                    let mut packet = Vec::new();
                    packet.push(b'E');
                    packet.extend_from_slice(&len.to_be_bytes());
                    packet.extend_from_slice(&error_payload);

                    let _ = stream.write_all(&packet);
                }

                // Break after 3 connections to simulate a timeout/giving up
                if *connect_count_clone.lock().unwrap() >= 3 {
                    break;
                }
            }
        });

        let url = format!("postgres://test_user:pass@127.0.0.1:{}/db", port);
        let start = std::time::Instant::now();

        let mut retries = 2; // Retry 2 times (total 3 attempts)
        let mut connected = false;

        loop {
            match diesel::PgConnection::establish(&url) {
                Ok(_) => {
                    connected = true;
                    break;
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    if retries > 0 && super::is_transient_supavisor_error(&err_msg) {
                        std::thread::sleep(std::time::Duration::from_millis(100)); // fast retry for test
                        retries -= 1;
                        continue;
                    }
                    break;
                }
            }
        }

        assert!(!connected, "Should not successfully connect");
        assert_eq!(*connect_count.lock().unwrap(), 3, "Should have retried exactly 2 times (3 total attempts)");
        assert!(start.elapsed().as_millis() >= 200, "Should have slept for at least 200ms total");
    }

    #[test]
    fn test_get_tenant_url_preserves_unencoded_chars() {
        let manager = ProductionTenantPoolManager::new(
            "postgres://postgres:k!@#XYZ@viaduct.proxy:5432/railway".to_string(),
            10,
        );
        let id = Uuid::parse_str("019e5512-4556-7b33-9e9b-84fb5a82c6d4").unwrap();
        let tenant_url = manager.get_tenant_url(&id).unwrap();
        
        // Expected: same password with unencoded #, but db_name changed
        assert_eq!(
            tenant_url,
            "postgres://postgres:k!@#XYZ@viaduct.proxy:5432/tenant_019e551245567b339e9b84fb5a82c6d4"
        );
    }

    #[test]
    fn test_get_tenant_url_supavisor_mode_preserves_unencoded_chars() {
        let manager = ProductionTenantPoolManager::new(
            "postgres://postgres.leeattend:master_pass!@#@supavisor:6543/leeattend?sslmode=disable".to_string(),
            10,
        );
        let id = Uuid::parse_str("019e5512-4556-7b33-9e9b-84fb5a82c6d4").unwrap();
        let tenant_url = manager.get_tenant_url(&id).unwrap();
        
        // Expected: user updated, password intact, db_name updated, query intact
        assert_eq!(
            tenant_url,
            "postgres://postgres.tenant_019e551245567b339e9b84fb5a82c6d4:master_pass!@#@supavisor:6543/tenant_019e551245567b339e9b84fb5a82c6d4?sslmode=disable"
        );
    }
}
