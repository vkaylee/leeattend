// Allow .unwrap() in test code — only production code is denied
#![cfg_attr(test, allow(clippy::unwrap_used))]

pub mod agent_manager;
pub mod cache;
pub mod db_pool;
pub mod dev_initialization;
pub mod domain;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod openapi;
pub mod queue;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod state;
pub mod utils;
pub mod worker;
pub mod mw;
pub mod app_builder;
#[cfg(test)]
mod workspace_id_guard;

use axum::Router;

use axum::http::{HeaderValue, Method};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub use crate::state::{
    AppState, AgentState, AttendanceState, CoreState, EmployeeState,
    InfraState, LeaveState, PayrollState, ReportState, ShiftState,
};

mod ax_auth {
    use crate::services::auth_service::AuthService;
    use crate::AppState;
    use axum::{
        async_trait,
        extract::{FromRef, FromRequestParts},
        http::{request::Parts, StatusCode},
    };
    use std::sync::Arc;
    use uuid::Uuid;

    pub struct UserId(pub Uuid);

    #[async_trait]
    impl<S> FromRequestParts<S> for UserId
    where
        S: Send + Sync,
        Arc<AppState>: FromRef<S>,
    {
        type Rejection = (StatusCode, String);

        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
            #[cfg(debug_assertions)]
            {
                if let Some(imp_id_str) = parts
                    .headers
                    .get("X-Debug-Impersonate-Id")
                    .and_then(|h| h.to_str().ok())
                {
                    if let Ok(user_id) = Uuid::parse_str(imp_id_str) {
                        return Ok(UserId(user_id));
                    }
                }
            }

            // 1. Try Authorization header first (Fallback for Native Apps / Postman)
            if let Some(auth_header) = parts.headers.get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        let app_state = Arc::<AppState>::from_ref(state);

                        if let Ok(user_id) = AuthService::verify_token(token, &app_state.core.jwt_secret) {
                            return Ok(UserId(user_id));
                        }
                    }
                }
            }

            // 2. Try Cookie header (Standard for Web Dashboard)
            if let Some(cookie_header) = parts.headers.get("cookie") {
                if let Ok(cookie_str) = cookie_header.to_str() {
                    for cookie in cookie_str.split(';') {
                        let cookie = cookie.trim();
                        if let Some(token) = cookie.strip_prefix("access_token=") {
                            let app_state = Arc::<AppState>::from_ref(state);
                            if let Ok(user_id) = AuthService::verify_token(token, &app_state.core.jwt_secret) {
                                return Ok(UserId(user_id));
                            }
                        }
                    }
                }
            }

            #[cfg(debug_assertions)]
            {
                if let Some(user_id_str) =
                    parts.headers.get("X-User-Id").and_then(|h| h.to_str().ok())
                {
                    if let Ok(user_id) = Uuid::parse_str(user_id_str) {
                        return Ok(UserId(user_id));
                    }
                }
            }

            Err((
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            ))
        }
    }
}

pub use ax_auth::UserId;

pub fn create_app(state: Arc<AppState>) -> Router {
    let allowed_origins_str = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "http://localhost:5173,http://localhost:3000,http://127.0.0.1:5173,http://127.0.0.1:3000".to_string()
        } else {
            "".to_string()
        }
    });

    let mut origins = Vec::new();
    for origin in allowed_origins_str.split(',') {
        let trimmed = origin.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(header_value) = trimmed.parse::<HeaderValue>() {
            origins.push(header_value);
        }
    }

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderName::from_static("x-requested-with"),
            axum::http::header::ACCEPT,
            axum::http::header::ORIGIN,
        ]);

    // Compose domain-specific route modules
    let router = Router::new()
        .merge(routes::dashboard::router())
        .merge(routes::auth::router(&state))
        .merge(routes::workspaces::router())
        .merge(routes::employees::router())
        .merge(routes::time_tracking::router())
        .merge(routes::leaves::router())
        .merge(routes::attendance::router())
        .merge(routes::reports::router(&state))
        .merge(routes::payroll::router())
        .merge(routes::shifts::router())
        .merge(routes::ess::router())
        .merge(routes::feature_flags::router());

    #[cfg(debug_assertions)]
    let router = {
        use axum::routing::{get, post};
        let router = router
            .route("/dev/super-user", get(handlers::get_dev_super_user))
            .route("/debug/users", get(handlers::debug::get_debug_users))
            .route(
                "/debug/workspaces/:ws_id/clear",
                post(handlers::debug::clear_debug_db),
            )
            .route(
                "/dev/workspaces/:ws_id/clear",
                post(handlers::clear_workspace_data),
            );
        tracing::warn!("⚠️ DEV MODE ACTIVE: Debug auth bypass & data-clearing endpoints are enabled. Never ship this to production!");
        router
    };

    Router::new()
        .nest("/api", router)
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

