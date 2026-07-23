//! HRM Attendance API Server entry point.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![allow(clippy::disallowed_methods)]

use apiserver::diesel_pool::{DbPoolConfig, DieselPools};
use apiserver::openapi::ApiDoc;
use apiserver::queue::RedisPool;
use apiserver::{create_app, AppState};
use mimalloc::MiMalloc;
use std::io::Write;
use tokio::time::timeout;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use uuid::Uuid;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

struct PortFileGuard {
    filename: String,
}

impl Drop for PortFileGuard {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.filename) {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::error!("Failed to remove {} file on shutdown: {}", self.filename, e);
            }
        } else {
            tracing::info!("{} file cleaned up successfully.", self.filename);
        }
    }
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for openapi generation flag before everything else
    if std::env::args().any(|arg| arg == "gen-openapi") {
        let openapi_json = ApiDoc::openapi().to_pretty_json()?;
        let mut stdout = std::io::stdout();
        let _ = stdout.write_all(openapi_json.as_bytes());
        let _ = stdout.write_all(b"\n");
        return Ok(());
    }

    // Fail-fast configuration validation
    let config = apiserver::utils::config::get_config();
    let app_env = &config.env;

    let registry = tracing_subscriber::registry()
        .with(apiserver::utils::logging::SpanContextLayer)
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "apiserver=debug,tower_http=debug".into()),
        );

    if app_env == "production" {
        registry
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(apiserver::utils::logging::ProductionJsonFormatter),
            )
            .init();
    } else {
        registry.with(tracing_subscriber::fmt::layer()).init();
    }

    // Log loaded configuration keys (no values)
    let mut loaded_keys = Vec::new();
    for key in &[
        "APP_ENV",
        "API_ENV",
        "APP_PORT",
        "API_PORT",
        "APP_HOST",
        "API_HOST",
        "DB_PRIMARY_URL",
        "DB_URL",
        "DATABASE_URL",
        "DB_POOLER_TX_URL",
        "DATABASE_POOLER_TX_URL",
        "DB_POOLER_SESS_URL",
        "DATABASE_POOLER_SESS_URL",
        "DB_READER_URL",
        "DATABASE_READER_URL",
        "REDIS_URL",
        "JWT_SECRET",
        "COOKIE_DOMAIN",
        "ALLOWED_ORIGINS",
        "ENCRYPTION_KEYS_JSON",
    ] {
        if std::env::var(key).is_ok() {
            loaded_keys.push(*key);
        }
    }
    tracing::info!(
        app_env = %app_env,
        loaded_keys = ?loaded_keys,
        "Configuration loaded successfully"
    );

    // Initialize configurable database pools (API + Worker + optional Reader)
    let db_config = DbPoolConfig::from_env()?;

    // Backup workspace configurations before dropping columns in system migrations
    let backup_cache = match tokio::task::spawn_blocking({
        let primary_url = db_config.primary_url.clone();
        move || {
            use diesel::Connection;
            use diesel::RunQueryDsl;

            #[derive(diesel::QueryableByName)]
            struct ExistsRow {
                #[diesel(sql_type = diesel::sql_types::Bool)]
                exists: bool,
            }

            #[derive(diesel::QueryableByName)]
            struct BackupRow {
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                id: Uuid,
                #[diesel(sql_type = diesel::sql_types::Text)]
                name: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                holiday_overnight_strategy: String,
                #[diesel(sql_type = diesel::sql_types::Numeric)]
                default_holiday_ot_multiplier: rust_decimal::Decimal,
            }

            let mut conn = diesel::PgConnection::establish(&primary_url)
                .map_err(|e| format!("Failed to connect to primary DB for backup: {e}"))?;

            let cols_exist = diesel::sql_query(
                "SELECT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='workspaces' AND column_name='holiday_overnight_strategy') as exists"
            )
            .get_result::<ExistsRow>(&mut conn)
            .map(|r| r.exists)
            .unwrap_or(false);

            if cols_exist {
                let rows = diesel::sql_query("SELECT id, name, holiday_overnight_strategy, default_holiday_ot_multiplier FROM workspaces")
                    .load::<BackupRow>(&mut conn)
                    .map_err(|e| format!("Failed to query workspaces backup: {e}"))?;

                let cache: std::collections::HashMap<Uuid, (String, String, rust_decimal::Decimal)> = rows
                    .into_iter()
                    .map(|r| (r.id, (r.name, r.holiday_overnight_strategy, r.default_holiday_ot_multiplier)))
                    .collect();
                Ok::<_, String>(cache)
            } else {
                Ok::<_, String>(std::collections::HashMap::new())
            }
        }
    })
    .await
    {
        Ok(Ok(cache)) => cache,
        _ => std::collections::HashMap::new(),
    };

    // 1. System migrations must run via direct connection before Supavisor is available
    DieselPools::run_system_migrations(&db_config)?;

    // 2. Connect pools (API/worker/tenant all go through PgBouncer)
    let diesel_pools = DieselPools::connect(&db_config)?;

    // 3. Run pending migrations & reconcile in the background
    let db_config_clone = db_config.clone();
    let diesel_pools_clone = diesel_pools.clone();
    tokio::spawn(async move {
        tracing::info!("Starting background tenant migrations and reconciliation...");
        if let Err(e) =
            apiserver::utils::startup_reconciliation::register_workspace_tenants_and_migrate(
                &db_config_clone,
                &diesel_pools_clone,
                backup_cache,
            )
            .await
        {
            tracing::error!("Error during background tenant migrations: {:?}", e);
        } else {
            tracing::info!(
                "Background tenant migrations and reconciliation completed successfully."
            );
        }
    });

    // ╔═══════════════════════════════════════════════════════════════╗
    // ║  POOL ASSIGNMENT RULES                                       ║
    // ║                                                               ║
    // ║  api pool    → HTTP handlers, CRUD, auth, repos in AppState   ║
    // ║  worker pool → background jobs, import, accrual, log flush    ║
    // ║  reader pool → dashboard analytics, ESS read-only queries     ║
    // ║                                                               ║
    // ║  ⚠  DO NOT pass worker_pool to API services                   ║
    // ║  ⚠  DO NOT use state.infra().db.reader() for mutations (INSERT/etc)   ║
    // ╚═══════════════════════════════════════════════════════════════╝

    // Initialize Redis for distributed job processing and presence tracking (optional)
    tracing::info!("Initializing Redis connection...");
    let redis_pool = init_redis().await;
    tracing::info!("Redis initialization complete");

    let (state, workers) = AppState::build(diesel_pools, redis_pool).await?;

    let app = create_app(std::sync::Arc::clone(&state));

    let host = std::env::var("APP_HOST")
        .or_else(|_| std::env::var("API_HOST"))
        .unwrap_or_else(|_| "0.0.0.0".to_string());
    let host = host.trim().to_string();
    let port_str = std::env::var("APP_PORT")
        .or_else(|_| std::env::var("API_PORT"))
        .unwrap_or_else(|_| "3000".to_string());
    let preferred_port = port_str.parse::<u16>().map_err(|e| {
        Box::<dyn std::error::Error>::from(format!("FATAL: Invalid APP_PORT/API_PORT: {e}"))
    })?;

    let port_file_name =
        std::env::var("PORT_FILE_NAME").unwrap_or_else(|_| ".api_port".to_string());
    let _ = std::fs::remove_file(&port_file_name);

    let listener = {
        let mut listener = None;
        let max_port = preferred_port + 10;

        for port in preferred_port..max_port {
            let addr_str = format!("{host}:{port}");
            match tokio::net::TcpListener::bind(&addr_str).await {
                Ok(l) => {
                    listener = Some(l);
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                    tracing::warn!("Port {} is in use, trying next port", port);
                }
                Err(e) => return Err(format!("Failed to bind to {addr_str}: {e}").into()),
            }
        }
        listener.ok_or("Could not find an available port")?
    };

    let actual_addr = listener.local_addr()?;
    if std::fs::write(&port_file_name, actual_addr.port().to_string()).is_ok() {
        tracing::info!("Actual listening on {}", actual_addr);
    }

    let _guard = PortFileGuard {
        filename: port_file_name,
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Graceful shutdown sequence
    tracing::info!("Starting graceful shutdown...");

    // Cancel worker tokens first
    workers.job_worker_token.cancel();
    if let Some(handle) = workers.job_worker_handle {
        tracing::info!("Waiting for job worker to complete...");
        if let Err(e) = handle.await {
            tracing::error!("Job worker task panicked: {:?}", e);
        }
    }
    if let Some(handle) = workers.outbox_sweep_handle {
        tracing::info!("Waiting for outbox sweep scheduler to complete...");
        if let Err(e) = handle.await {
            tracing::error!("Outbox sweep scheduler task panicked: {:?}", e);
        }
    }

    state.agents().manager().shutdown().await;

    drop(state);

    tracing::info!("Server stopped.");
    Ok(())
}

async fn init_redis() -> Option<RedisPool> {
    let Ok(url) = std::env::var("REDIS_URL") else {
        tracing::info!("REDIS_URL not set, distributed processing and presence tracking disabled");
        return None;
    };

    tracing::info!("Connecting to Redis at {}", url);

    let pool = match timeout(std::time::Duration::from_secs(5), RedisPool::new(&url)).await {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => {
            tracing::warn!("Redis connection failed: {}", e);
            return None;
        }
        Err(_) => {
            tracing::warn!("Redis connection timed out");
            return None;
        }
    };

    tracing::info!("Redis pool created, checking health...");
    match timeout(std::time::Duration::from_secs(5), pool.health_check()).await {
        Ok(Ok(())) => {
            tracing::info!("Redis connected for distributed processing and presence");
            Some(pool)
        }
        Ok(Err(e)) => {
            tracing::warn!("Redis health check failed: {}", e);
            None
        }
        Err(_) => {
            tracing::warn!("Redis health check timed out");
            None
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            tracing::error!("failed to install Ctrl+C handler: {}", err);
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => {
                tracing::error!("failed to install signal handler: {}", err);
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
