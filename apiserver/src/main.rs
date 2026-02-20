use apiserver::openapi::ApiDoc;
use apiserver::queue::RedisPool;
use apiserver::services::{AttendanceImportService, AttendanceLogService, ReportCoordinator};
use apiserver::worker::{ReportWorker, WorkerConfig};
use apiserver::{agent_manager::AgentManager, create_app, AppState};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;

struct PortFileGuard {
    filename: String,
}

impl Drop for PortFileGuard {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.filename) {
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!("Failed to remove {} file on shutdown: {}", self.filename, e);
            }
        } else {
            eprintln!("{} file cleaned up successfully.", self.filename);
        }
    }
}

#[tokio::main]
async fn main() {
    // Check for openapi generation flag before everything else
    if std::env::args().any(|arg| arg == "gen-openapi") {
        let openapi_json = ApiDoc::openapi()
            .to_pretty_json()
            .expect("Failed to generate OpenAPI JSON");
        println!("{openapi_json}");
        return;
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "apiserver=debug,tower_http=debug,sqlx=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    AttendanceImportService::resume_all_pending_jobs(pool.clone()).await;

    #[cfg(debug_assertions)]
    {
        apiserver::dev_initialization::ensure_dev_data(&pool)
            .await
            .expect("Failed to seed dev data");
    }

    // Initialize Redis for distributed job processing and presence tracking (optional)
    tracing::info!("Initializing Redis connection...");
    let redis_pool = match std::env::var("REDIS_URL") {
        Ok(url) => {
            tracing::info!("Connecting to Redis at {}", url);
            match timeout(std::time::Duration::from_secs(5), RedisPool::new(&url)).await {
                Ok(pool_result) => match pool_result {
                    Ok(pool) => {
                        // Verify connection with health check
                        tracing::info!("Redis pool created, checking health...");
                        match timeout(std::time::Duration::from_secs(5), pool.health_check()).await
                        {
                            Ok(Ok(())) => {
                                tracing::info!(
                                    "Redis connected for distributed processing and presence"
                                );
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
                    Err(e) => {
                        tracing::warn!("Redis connection failed: {}", e);
                        None
                    }
                },
                Err(_) => {
                    tracing::warn!("Redis connection timed out");
                    None
                }
            }
        }
        Err(_) => {
            tracing::info!(
                "REDIS_URL not set, distributed processing and presence tracking disabled"
            );
            None
        }
    };
    tracing::info!("Redis initialization complete");

    let agent_manager = AgentManager::new(
        redis_pool
            .clone()
            .map(|p| Arc::new(p) as Arc<dyn apiserver::agent_manager::PresenceStore>),
    );
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret_key_leeattend_2026".to_string());

    let buffer_capacity: usize = std::env::var("AGENT_LOG_BUFFER_CAPACITY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5000);
    let batch_size: usize = std::env::var("AGENT_LOG_BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let flush_interval_ms: u64 = std::env::var("AGENT_LOG_FLUSH_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);

    let (attendance_service, log_worker_handle) =
        AttendanceLogService::new(pool.clone(), buffer_capacity, batch_size, flush_interval_ms);
    let attendance_service = Arc::new(attendance_service);

    // Roster cache (requires Redis; shared between app state and worker)
    let roster_cache = redis_pool
        .as_ref()
        .map(|rpool| Arc::new(apiserver::cache::RosterCache::new(rpool.clone())));

    // Initialize report worker if Redis is available
    let worker_cancel_token = CancellationToken::new();
    let (coordinator, worker_handle) = if let Some(ref redis) = redis_pool {
        let worker_config = WorkerConfig::from_env();
        tracing::info!(
            max_concurrent = worker_config.max_concurrent_jobs,
            poll_timeout = worker_config.poll_timeout_secs,
            "Initializing report worker"
        );

        let coordinator = Arc::new(ReportCoordinator::new(redis.clone(), pool.clone()));

        let worker = ReportWorker::new(
            redis.clone(),
            pool.clone(),
            worker_config,
            coordinator.clone(),
            roster_cache.clone(),
        );

        (
            Some(coordinator),
            Some(worker.start(worker_cancel_token.clone())),
        )
    } else {
        (None, None)
    };

    let state = Arc::new(AppState {
        agent_manager,
        db: pool,
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret,
        attendance_service,
        redis: redis_pool,
        coordinator,
        roster_cache,
    });

    let app = create_app(state.clone());

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
        let max_port = preferred_port + 100;

        for port in preferred_port..max_port {
            let addr_str = format!("{host}:{port}");
            match tokio::net::TcpListener::bind(&addr_str).await {
                Ok(l) => {
                    listener = Some(l);
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                    tracing::warn!("Port {} is in use, trying next port", port);
                    continue;
                }
                Err(e) => panic!("Failed to bind to {addr_str}: {e}"),
            }
        }
        listener.expect("Could not find an available port")
    };

    let actual_addr = listener
        .local_addr()
        .expect("Failed to get listener local address");
    if std::fs::write(&port_file_name, actual_addr.port().to_string()).is_ok() {
        tracing::info!("Actual listening on {}", actual_addr);
    }

    let _guard = PortFileGuard {
        filename: port_file_name,
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server exited with error");

    // Graceful shutdown sequence
    tracing::info!("Starting graceful shutdown...");

    // Cancel worker tokens first
    worker_cancel_token.cancel();
    if let Some(handle) = worker_handle {
        tracing::info!("Waiting for report worker to complete...");
        if let Err(e) = handle.await {
            tracing::error!("Report worker task panicked: {:?}", e);
        }
    }

    state.agent_manager.shutdown().await;

    drop(state);

    tracing::info!("Waiting for log worker to drain...");
    if let Err(e) = log_worker_handle.await {
        tracing::error!("Log worker task panicked: {:?}", e);
    }

    tracing::info!("Server stopped.");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
