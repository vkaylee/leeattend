//! HRM Attendance API Server entry point.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

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

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "apiserver=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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

    // 2. Initialize Supavisor pooler — system tenant must be registered before pools connect
    let tenant_manager: std::sync::Arc<dyn apiserver::supavisor::TenantManager> =
        match apiserver::supavisor::SupavisorTenantManager::new() {
            Ok(mgr) => std::sync::Arc::new(mgr),
            Err(e) => {
                if cfg!(debug_assertions) {
                    tracing::warn!("⚠️ Supavisor variables not fully set ({:?}) — falling back to NoopTenantManager for development.", e);
                    std::sync::Arc::new(apiserver::supavisor::NoopTenantManager)
                } else {
                    return Err(Box::<dyn std::error::Error>::from(e));
                }
            }
        };
    tenant_manager.init_tenant().await;

    // 3. Connect pools (API/worker/tenant all go through Supavisor)
    let diesel_pools = DieselPools::connect(&db_config, std::sync::Arc::clone(&tenant_manager))?;

    // 4. Register existing workspace tenants with Supavisor, then run pending migrations & reconcile
    register_workspace_tenants_and_migrate(
        &db_config,
        &diesel_pools,
        &*tenant_manager,
        backup_cache,
    )
    .await?;

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

    let (state, workers) = AppState::build(diesel_pools, redis_pool, tenant_manager).await?;

    let app = create_app(std::sync::Arc::clone(&state));

    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let host = host.trim().to_string();
    let preferred_port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

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

    tracing::info!("Waiting for log worker to drain...");
    if let Err(e) = workers.log_worker_handle.await {
        tracing::error!("Log worker task panicked: {:?}", e);
    }

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

/// Register all existing workspace tenants with Supavisor, then run pending
/// tenant migrations. Called once at startup after pools are connected.
#[allow(clippy::too_many_lines)]
async fn register_workspace_tenants_and_migrate(
    db_config: &DbPoolConfig,
    diesel_pools: &DieselPools,
    tenant_manager: &dyn apiserver::supavisor::TenantManager,
    backup_cache: std::collections::HashMap<Uuid, (String, String, rust_decimal::Decimal)>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Query workspaces from system DB via direct connection
    let primary_url = db_config.primary_url.clone();
    let workspaces: Vec<(Uuid, Option<String>)> = match tokio::task::spawn_blocking(move || {
        use diesel::sql_types::{Nullable, Text, Uuid as UuidSql};
        use diesel::{Connection, QueryableByName, RunQueryDsl};

        #[derive(QueryableByName)]
        struct WsRow {
            #[diesel(sql_type = UuidSql)]
            id: Uuid,
            #[diesel(sql_type = Nullable<Text>)]
            custom_db_url_encrypted: Option<String>,
        }

        let mut conn = diesel::PgConnection::establish(&primary_url)
            .map_err(|e| format!("Failed to connect: {e}"))?;

        let workspaces = diesel::sql_query("SELECT id, custom_db_url_encrypted FROM workspaces")
            .load::<WsRow>(&mut conn)
            .map(|r| {
                r.into_iter()
                    .map(|w| (w.id, w.custom_db_url_encrypted))
                    .collect()
            })
            .map_err(|e| format!("Failed to query workspaces: {e}"))?;

        Ok::<_, String>(workspaces)
    })
    .await
    {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            tracing::warn!("{e} — skipping tenant registration");
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to spawn blocking task for workspace query: {e}");
            return Ok(());
        }
    };

    if workspaces.is_empty() {
        tracing::info!("No workspaces — skipping tenant registration and migrations");
        return Ok(());
    }

    let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let crypto_service: Option<apiserver::crypto::CryptoService> =
        if let Ok(keys) = std::env::var("ENCRYPTION_KEYS_JSON") {
            let crypto = apiserver::crypto::CryptoService::from_json(&keys)
                .map_err(|e| format!("Invalid ENCRYPTION_KEYS_JSON: {e}"))?;
            Some(crypto)
        } else if app_env == "production" {
            return Err(Box::from("ENCRYPTION_KEYS_JSON must be set in production"));
        } else {
            None
        };

    let mut migration_targets: Vec<(Uuid, String)> = Vec::new();

    for (ws_id, custom_db_url_encrypted) in workspaces {
        let db_name = format!("tenant_{}", ws_id.to_string().replace('-', ""));

        let tenant_url = if let Some(encrypted) = custom_db_url_encrypted {
            if let Some(crypto) = &crypto_service {
                crypto.decrypt(&encrypted).unwrap_or(encrypted)
            } else {
                encrypted
            }
        } else {
            tracing::warn!(
                "Workspace {} missing custom_db_url_encrypted! Skipping registration.",
                ws_id
            );
            continue;
        };

        let Some((_, payload)) = apiserver::supavisor::build_tenant_payload(
            &tenant_url,
            apiserver::supavisor::TenantPayloadOptions {
                tenant_id: Some(db_name.clone()),
                pool_size: None,
                is_master: false,
            },
        ) else {
            tracing::warn!("Failed to build tenant payload for {}", db_name);
            continue;
        };

        if let Err(e) = tenant_manager
            .register_tenant_dynamic(&db_name, &payload)
            .await
        {
            tracing::warn!(
                workspace_id = %ws_id,
                error = %e,
                "Failed to register tenant with Supavisor — will retry on next startup"
            );
            continue;
        }

        // Resolve migration URL via the session pool manager — it knows whether
        // to route through Supavisor (prod) or connect directly (dev), and uses
        // the tenant's own credentials extracted from `tenant_url`.
        let migration_url = match diesel_pools
            .tenants_sess
            .resolve_migration_url(&ws_id, &tenant_url)
        {
            Ok(url) => url,
            Err(e) => {
                tracing::warn!(
                    workspace_id = %ws_id,
                    error = %e,
                    "Failed to resolve tenant migration URL — skipping"
                );
                continue;
            }
        };

        migration_targets.push((ws_id, migration_url));
    }

    if !migration_targets.is_empty() {
        tracing::info!(
            workspace_count = migration_targets.len(),
            "Running tenant migrations with per-tenant credentials"
        );

        let cache_clone = backup_cache.clone();
        match tokio::task::spawn_blocking(move || {
            for (ws_id, migration_url) in &migration_targets {
                if let Err(e) = DieselPools::run_single_tenant_migration(migration_url, ws_id) {
                    tracing::warn!(
                        workspace_id = %ws_id,
                        error = %e,
                        "Tenant migration failed — skipping workspace"
                    );
                    continue;
                }

                // Run EAV reconciliation
                let backup_info = cache_clone.get(ws_id);
                if let Err(e) = reconcile_tenant_database(migration_url, ws_id, backup_info) {
                    tracing::error!(
                        workspace_id = %ws_id,
                        error = %e,
                        "Failed to reconcile tenant database: {e}"
                    );
                }
            }
        })
        .await
        {
            Ok(()) => {}
            Err(e) => tracing::error!("Tenant migration task panicked: {e}"),
        }
    }
    Ok(())
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

fn reconcile_tenant_database(
    migration_url: &str,
    ws_id: &Uuid,
    backup_info: Option<&(String, String, rust_decimal::Decimal)>,
) -> Result<(), String> {
    use diesel::Connection;
    use diesel::RunQueryDsl;

    #[derive(diesel::QueryableByName)]
    struct ExistsRow {
        #[diesel(sql_type = diesel::sql_types::Bool)]
        exists: bool,
    }

    let mut conn = diesel::PgConnection::establish(migration_url)
        .map_err(|e| format!("Failed to connect to tenant database: {e}"))?;

    // 1. Seed overtime.holiday_overnight_strategy
    let strategy_exists: bool = diesel::sql_query(
        "SELECT EXISTS(SELECT 1 FROM workspace_config_entries WHERE workspace_id = $1 AND scope_type = 'workspace' AND scope_id IS NULL AND config_key = 'overtime.holiday_overnight_strategy')"
    )
    .bind::<diesel::sql_types::Uuid, _>(*ws_id)
    .get_result::<ExistsRow>(&mut conn)
    .map(|r| r.exists)
    .map_err(|e| format!("Failed to check strategy existence: {e}"))?;

    if !strategy_exists {
        let strategy_val = if let Some((_, strategy, _)) = backup_info {
            strategy.clone()
        } else {
            "split_at_midnight".to_string()
        };

        diesel::sql_query(
            "INSERT INTO workspace_config_entries (id, workspace_id, scope_type, scope_id, config_key, value_text, effective_from) \
             VALUES (gen_random_uuid(), $1, 'workspace', NULL, 'overtime.holiday_overnight_strategy', $2, '1970-01-01')"
        )
        .bind::<diesel::sql_types::Uuid, _>(*ws_id)
        .bind::<diesel::sql_types::Text, _>(strategy_val)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to insert default strategy: {e}"))?;
    }

    // 2. Seed overtime.default_holiday_ot_multiplier
    let multiplier_exists: bool = diesel::sql_query(
        "SELECT EXISTS(SELECT 1 FROM workspace_config_entries WHERE workspace_id = $1 AND scope_type = 'workspace' AND scope_id IS NULL AND config_key = 'overtime.default_holiday_ot_multiplier')"
    )
    .bind::<diesel::sql_types::Uuid, _>(*ws_id)
    .get_result::<ExistsRow>(&mut conn)
    .map(|r| r.exists)
    .map_err(|e| format!("Failed to check multiplier existence: {e}"))?;

    if !multiplier_exists {
        let multiplier_val = if let Some((_, _, multiplier)) = backup_info {
            *multiplier
        } else {
            rust_decimal::Decimal::new(30, 1) // 3.0
        };

        diesel::sql_query(
            "INSERT INTO workspace_config_entries (id, workspace_id, scope_type, scope_id, config_key, value_decimal, effective_from) \
             VALUES (gen_random_uuid(), $1, 'workspace', NULL, 'overtime.default_holiday_ot_multiplier', $2, '1970-01-01')"
        )
        .bind::<diesel::sql_types::Uuid, _>(*ws_id)
        .bind::<diesel::sql_types::Numeric, _>(multiplier_val)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to insert default multiplier: {e}"))?;
    }

    Ok(())
}
