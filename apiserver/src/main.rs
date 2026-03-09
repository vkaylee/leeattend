use apiserver::db_pool::{DbPoolConfig, DbPools};
use apiserver::openapi::ApiDoc;
use apiserver::queue::RedisPool;
use apiserver::{create_app, AppState};
use tokio::time::timeout;
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

    // Initialize configurable database pools (API + Worker + optional Reader)
    let db_config = DbPoolConfig::from_env();
    let db_pools = DbPools::connect(&db_config).await;
    db_pools.run_migrations().await;

    // ╔═══════════════════════════════════════════════════════════════╗
    // ║  POOL ASSIGNMENT RULES                                       ║
    // ║                                                               ║
    // ║  api pool    → HTTP handlers, CRUD, auth, repos in AppState   ║
    // ║  worker pool → background jobs, import, accrual, log flush    ║
    // ║  reader pool → dashboard analytics, ESS read-only queries     ║
    // ║                                                               ║
    // ║  ⚠  DO NOT pass worker_pool to API services                   ║
    // ║  ⚠  DO NOT use state.infra.db.reader() for mutations (INSERT/etc)   ║
    // ╚═══════════════════════════════════════════════════════════════╝

    // Initialize Redis for distributed job processing and presence tracking (optional)
    tracing::info!("Initializing Redis connection...");
    let redis_pool = init_redis().await;
    tracing::info!("Redis initialization complete");

    let (state, workers) = AppState::build(db_pools, redis_pool).await;

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
    workers.job_worker_token.cancel();
    if let Some(handle) = workers.job_worker_handle {
        tracing::info!("Waiting for job worker to complete...");
        if let Err(e) = handle.await {
            tracing::error!("Job worker task panicked: {:?}", e);
        }
    }

    state.agents.manager.shutdown().await;

    drop(state);

    tracing::info!("Waiting for log worker to drain...");
    if let Err(e) = workers.log_worker_handle.await {
        tracing::error!("Log worker task panicked: {:?}", e);
    }

    tracing::info!("Server stopped.");
}

async fn init_redis() -> Option<RedisPool> {
    let url = match std::env::var("REDIS_URL") {
        Ok(u) => u,
        Err(_) => {
            tracing::info!("REDIS_URL not set, distributed processing and presence tracking disabled");
            return None;
        }
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
