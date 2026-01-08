pub mod agent_manager;
pub mod handlers;
pub mod models;

use axum::{
    routing::{get, post, put},
    Router,
};

use crate::agent_manager::AgentManager;
use axum::http::{HeaderValue, Method};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub struct AppState {
    pub agent_manager: AgentManager,
    pub db: PgPool,
}

pub fn create_app(state: Arc<AppState>) -> Router {
    // Cấu hình CORS từ biến môi trường
    let allowed_origins_str =
        std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "http://localhost:3001".to_string());

    let mut origins = Vec::new();
    for origin in allowed_origins_str.split(',') {
        let trimmed = origin.trim();
        if trimmed.is_empty() {
            continue;
        }

        match trimmed.parse::<HeaderValue>() {
            Ok(header_value) => {
                origins.push(header_value);
                tracing::info!("CORS: Allowed origin added: {}", trimmed);
            }
            Err(e) => {
                tracing::error!("CORS: Failed to parse origin '{}': {}", trimmed, e);
            }
        }
    }

    if origins.is_empty() {
        tracing::warn!("CORS: No valid origins found. All cross-origin requests will be rejected!");
    }

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_headers(Any);

    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/ws", get(handlers::ws_handler))
        .route("/agents", get(handlers::get_agents))
        .route("/agent/:agent_id/command", post(handlers::send_command))
        // Device Management
        .route(
            "/devices",
            get(handlers::get_devices).post(handlers::create_device),
        )
        .route(
            "/devices/:id",
            put(handlers::update_device).delete(handlers::delete_device),
        )
        // Authorized Agents
        .route(
            "/authorized-agents",
            get(handlers::get_authorized_agents).post(handlers::create_authorized_agent),
        )
        .route(
            "/authorized-agents/:id",
            get(handlers::get_authorized_agent_by_id)
                .patch(handlers::update_authorized_agent)
                .delete(handlers::delete_authorized_agent),
        )
        .route(
            "/authorized-agents/:id/trust",
            put(handlers::update_agent_trust),
        )
        // Workspace Management
        .route(
            "/workspaces",
            get(handlers::get_workspaces).post(handlers::create_workspace),
        )
        .route(
            "/workspaces/:ws_id/members",
            get(handlers::get_workspace_members).post(handlers::add_workspace_member),
        )
        .layer(cors)
        .with_state(state)
}
