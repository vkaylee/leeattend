use apiserver::{agent_manager::AgentManager, create_app, AppState};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing (logging)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "apiserver=debug,tower_http=debug,sqlx=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Initialize AppState
    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret_key_leeattend_2026".to_string());

    let state = Arc::new(AppState {
        agent_manager,
        db: pool,
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret,
    });

    // Build our application
    let app = create_app(state.clone());

    // Read host and port from environment variables
    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let preferred_port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    // Try to bind to the preferred port, or find another one if it's busy
    let listener = {
        let mut listener = None;
        let max_port = preferred_port + 100; // Try up to 100 next ports
        
        for port in preferred_port..max_port {
            let addr_str = format!("{}:{}", host, port);
            match tokio::net::TcpListener::bind(&addr_str).await {
                Ok(l) => {
                    listener = Some(l);
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                    tracing::warn!("Port {} is already in use, trying next one...", port);
                    continue;
                }
                Err(e) => panic!("Failed to bind to {}: {}", addr_str, e),
            }
        }
        
        listener.expect("Could not find an available port in the range 3000-3100. Throwing exception.")
    };

    let actual_addr = listener.local_addr().unwrap();
    tracing::info!("Actual listening on {}", actual_addr);

    // Ghi port vào file để Frontend/AI có thể tự động khám phá
    if let Ok(_) = std::fs::write(".api_port", actual_addr.port().to_string()) {
        tracing::info!("Saved actual port to .api_port: {}", actual_addr.port());
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    // Cleanup port file on exit
    let _ = std::fs::remove_file(".api_port");

    // Sau khi server ngừng nhận request, thực hiện thông báo cho Agent và đóng tài nguyên
    state.agent_manager.shutdown().await;
    state.db.close().await;

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

    tracing::info!("Graceful shutdown signal received.");
}
