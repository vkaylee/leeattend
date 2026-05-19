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
        let mut parsed_url = reqwest::Url::parse(&self.base_url)
            .map_err(|e| AppError::Internal(format!("Invalid base pool URL: {e}")))?;
        parsed_url.set_path(&db_name);
        Ok(parsed_url.to_string())
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

impl DieselPools {
    /// Convenience method: get a connection from the tenant pool for a given workspace.
    pub async fn tenant_conn(&self, workspace_id: Uuid) -> Result<DieselConnection, AppError> {
        let pool = self.tenants.get_pool(&workspace_id).await?;
        pool.get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
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

        let base_url = &config.primary_url;

        for row in &workspace_rows {
            let db_name = format!("tenant_{}", row.id.to_string().replace('-', ""));
            let tenant_url = match reqwest::Url::parse(base_url) {
                Ok(mut url) => {
                    url.set_path(&db_name);
                    url.to_string()
                }
                Err(e) => {
                    tracing::error!(
                        workspace_id = %row.id,
                        error = %e,
                        "Failed to parse base URL for tenant migrations — skipping workspace"
                    );
                    continue;
                }
            };

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
                }
                Err(e) => {
                    tracing::warn!(
                        workspace_id = %row.id,
                        error = %e,
                        "Failed to connect to tenant DB — may not exist yet, skipping"
                    );
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
    fn test_tenant_pool_manager_url_production_mode() {
        let base_url = "postgres://user:pass@localhost:5432/postgres";
        let manager = ProductionTenantPoolManager::new(base_url.to_string(), 5);

        // Use a fixed UUID for predictable output
        let workspace_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();

        let result = manager.get_tenant_url(&workspace_id).unwrap();

        // In production mode, it should append the tenant name to the path
        assert_eq!(
            result,
            "postgres://user:pass@localhost:5432/tenant_123e4567e89b12d3a456426614174000"
        );
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
}
