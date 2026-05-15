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

#[derive(Clone)]
pub struct DieselPools {
    pub api: DieselPool,
    pub worker: DieselPool,
}

impl DieselPools {
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

        tracing::info!("Diesel pools connected successfully");

        Self {
            api: api_pool,
            worker: worker_pool,
        }
    }

    #[allow(clippy::expect_used)]
    pub fn run_migrations(config: &DbPoolConfig) {
        use diesel::Connection;
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

        let mut conn = diesel::PgConnection::establish(&config.primary_url)
            .expect("Failed to connect to primary database for migrations");

        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        tracing::info!("Database migrations completed successfully");
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

        Self {
            api: api_pool,
            worker: worker_pool,
        }
    }
}
