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
#[allow(clippy::disallowed_macros)] // eprintln! needed before tracing init for fatal config errors
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for env-example generation flag before everything else
    if std::env::args().any(|arg| arg == "gen-env-example") {
        let example = apiserver::utils::env_registry::EnvRegistry::generate_env_example();
        let mut stdout = std::io::stdout();
        let _ = stdout.write_all(example.as_bytes());
        return Ok(());
    }

    // Check for openapi generation flag before everything else
    if std::env::args().any(|arg| arg == "gen-openapi") {
        let openapi_json = ApiDoc::openapi().to_pretty_json()?;
        let mut stdout = std::io::stdout();
        let _ = stdout.write_all(openapi_json.as_bytes());
        let _ = stdout.write_all(b"\n");
        return Ok(());
    }

    // Run the read-only qualifying-day diagnostic through the runtime tenant pool manager.
    if apiserver::diagnostics::qualifying_days_blast_radius::requested() {
        return run_qualifying_days_blast_radius().await;
    }

    // ╔═══════════════════════════════════════════════════════════════╗
    // ║  ENVIRONMENT CONFIGURATION — centralized registry            ║
    // ╚═══════════════════════════════════════════════════════════════╝
    // Resolve the centralized env registry and emit the startup summary
    // BEFORE initializing tracing, so any resolution errors are visible
    // on stderr/stdout.
    let registry =
        apiserver::utils::env_registry::EnvRegistry::resolve().unwrap_or_else(|errors| {
            eprintln!("╔══════════════════════════════════════════════════════════════╗");
            eprintln!(
                "║  ❌ CONFIGURATION FAILURE — {} missing env var(s)      ║",
                errors.len()
            );
            eprintln!("╠══════════════════════════════════════════════════════════════╣");
            for err in &errors {
                eprintln!("║  • {err}");
            }
            eprintln!("╚══════════════════════════════════════════════════════════════╝");
            std::process::exit(1);
        });

    // Fail-fast configuration validation (uses OnceLock, reads from env)
    let config = apiserver::utils::config::get_config();
    let app_env = &config.env;

    let registry_for_tracing = tracing_subscriber::registry()
        .with(apiserver::utils::logging::SpanContextLayer)
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "apiserver=debug,tower_http=debug".into()),
        );

    if app_env == "production" {
        registry_for_tracing
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(apiserver::utils::logging::ProductionJsonFormatter),
            )
            .init();
    } else {
        registry_for_tracing
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    // Emit the full startup configuration summary
    tracing::info!("{}", registry.format_summary());

    // Initialize configurable database pools (API + Worker + optional Reader)
    let db_config = DbPoolConfig::from_registry(&registry)?;

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
    let redis_pool = init_redis(&registry).await?;
    tracing::info!("Redis initialization complete");

    let (state, workers) = AppState::build(diesel_pools, redis_pool, &registry).await?;

    let app = create_app(std::sync::Arc::clone(&state), &registry);

    // ── Bind to port (from AppConfig, already validated) ──────────────────

    let host = config.host.clone();
    let preferred_port = config.port;
    let port_file_name = config.port_file_name.clone();
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

#[allow(clippy::disallowed_macros)]
async fn run_qualifying_days_blast_radius() -> Result<(), Box<dyn std::error::Error>> {
    let (workspace_id, timezone, from_date, to_date) =
        apiserver::diagnostics::qualifying_days_blast_radius::parse_process_args()?;
    let registry = apiserver::utils::env_registry::EnvRegistry::resolve().map_err(|errors| {
        format!(
            "configuration failure: {} missing variable(s)",
            errors.len()
        )
    })?;
    let db_config = DbPoolConfig::from_registry(&registry)?;
    let pools = DieselPools::connect(&db_config)?;
    let report = apiserver::diagnostics::qualifying_days_blast_radius::run(
        &pools,
        workspace_id,
        &timezone,
        from_date,
        to_date,
    )
    .await?;

    println!("Qualifying-day blast-radius report (read-only)");
    println!("Workspace: {}", report.workspace_id);
    println!("Range: {} through {}", report.from_date, report.to_date);
    println!(
        "Affected employee-months: {}",
        report.affected_employee_months
    );
    println!("Candidate inflated days: {}", report.total_inflated_days);
    println!("Current batch-style days: {}", report.current_batch_days);
    println!(
        "Corrected qualifying days: {}",
        report.corrected_qualifying_days
    );
    Ok(())
}

async fn init_redis(
    registry: &apiserver::utils::env_registry::EnvRegistry,
) -> Result<Option<RedisPool>, Box<dyn std::error::Error>> {
    let Some(url) = registry.get("REDIS_URL") else {
        tracing::info!("REDIS_URL not set, distributed processing and presence tracking disabled");
        return Ok(None);
    };

    tracing::info!("Connecting to configured Redis service");

    let pool = match timeout(std::time::Duration::from_secs(5), RedisPool::new(url)).await {
        Ok(Ok(pool)) => pool,
        Ok(Err(error)) => {
            tracing::error!(error = ?error, "Redis connection failed; refusing to start");
            return Err("Redis is configured but unavailable".into());
        }
        Err(_) => {
            tracing::error!("Redis connection timed out; refusing to start");
            return Err("Redis is configured but unavailable".into());
        }
    };

    tracing::info!("Redis pool created, checking health...");
    match timeout(std::time::Duration::from_secs(5), pool.health_check()).await {
        Ok(Ok(())) => {
            tracing::info!("Redis connected for distributed processing and presence");
            Ok(Some(pool))
        }
        Ok(Err(error)) => {
            tracing::error!(error = ?error, "Redis health check failed; refusing to start");
            Err("Redis is configured but unavailable".into())
        }
        Err(_) => {
            tracing::error!("Redis health check timed out; refusing to start");
            Err("Redis is configured but unavailable".into())
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
