pub mod agent_manager;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

use crate::agent_manager::AgentManager;
use crate::repositories::{AgentRepository, WorkspaceRepository};
use crate::services::AgentService;
use axum::http::{HeaderValue, Method};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub struct AppState {
    pub agent_manager: AgentManager,
    pub db: PgPool,
    pub agent_repo: Arc<dyn AgentRepository>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub agent_service: Arc<AgentService>,
    pub jwt_secret: String,
}

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
            // 1. Check for JWT in Authorization header
            if let Some(auth_header) = parts.headers.get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = &auth_str[7..];
                        let app_state = Arc::<AppState>::from_ref(state);

                        match AuthService::verify_token(token, &app_state.jwt_secret) {
                            Ok(user_id) => return Ok(UserId(user_id)),
                            Err(_) => {
                                return Err((
                                    StatusCode::UNAUTHORIZED,
                                    "Invalid or expired token".to_string(),
                                ))
                            }
                        }
                    }
                }
            }

            // 2. Fallback to X-User-Id in development (debug assertions)
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
    // Cấu hình CORS
    let cors = if cfg!(debug_assertions) {
        // Trong môi trường dev, cho phép tất cả để tránh phiền phức về port
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(Any)
    } else {
        // Trong môi trường production, cấu hình nghiêm ngặt từ env
        let allowed_origins_str =
            std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "".to_string());

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

        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
            ])
            .allow_headers(Any)
    };

    let mut router = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/auth/login", post(handlers::login))
        .route("/auth/register", post(handlers::register))
        .route("/auth/refresh", post(handlers::refresh_token));

    #[cfg(debug_assertions)]
    {
        router = router.route("/dev/super-user", get(handlers::get_dev_super_user));
    }

    router
        .route("/ws", get(handlers::ws_handler))
        .route("/agents", get(handlers::get_agents))
        .route("/agent/:agent_id/command", post(handlers::send_command))
        // Profile
        .route("/me", get(handlers::get_me).patch(handlers::update_me))
        .route("/auth/change-password", post(handlers::change_password))
        .route("/users", get(handlers::get_users))
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
        .route(
            "/workspaces/:ws_id/members/:user_id",
            patch(handlers::update_workspace_member_role).delete(handlers::remove_workspace_member),
        )
        .route(
            "/workspaces/:ws_id/members/:user_id/roles",
            get(handlers::get_member_roles).post(handlers::assign_extra_role),
        )
        .route(
            "/workspaces/:ws_id/members/:user_id/roles/:role_id",
            delete(handlers::revoke_extra_role),
        )
        // Employee Management
        .route(
            "/workspaces/:ws_id/employees",
            get(handlers::get_employees).post(handlers::create_employee),
        )
        .route(
            "/workspaces/:ws_id/employees/:id",
            get(handlers::get_employee)
                .put(handlers::update_employee)
                .delete(handlers::delete_employee),
        )
        // Time Tracking
        .route(
            "/workspaces/:ws_id/time/punch-in",
            post(handlers::punch_in),
        )
        .route(
            "/workspaces/:ws_id/time/punch-out/:employee_id",
            post(handlers::punch_out),
        )
        .route(
            "/workspaces/:ws_id/time/history/:employee_id",
            get(handlers::get_time_history),
        )
        .route(
            "/workspaces/:ws_id/time/entries/:id",
            put(handlers::update_time_entry).delete(handlers::delete_time_entry),
        )
        // Leave Management
        .route(
            "/workspaces/:ws_id/leave/submit",
            post(handlers::submit_leave_request),
        )
        .route(
            "/workspaces/:ws_id/leave/requests",
            get(handlers::get_workspace_leave_requests),
        )
        .route(
            "/workspaces/:ws_id/leave/requests/:id/approve",
            post(handlers::approve_leave_request),
        )
        .route(
            "/workspaces/:ws_id/leave/employee/:employee_id",
            get(handlers::get_my_leave_requests),
        )
        // Reporting
        .route(
            "/workspaces/:ws_id/reports/attendance",
            get(handlers::get_attendance_report),
        )
        // Payroll
        .route(
            "/workspaces/:ws_id/employees/:id/salary",
            get(handlers::get_salary_settings).put(handlers::update_salary_settings),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/payslips",
            get(handlers::get_employee_payslips),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/calculate-payroll",
            post(handlers::calculate_payroll),
        )
        // Shifts
        .route(
            "/workspaces/:ws_id/shifts",
            get(handlers::get_shifts).post(handlers::create_shift),
        )
        .route(
            "/workspaces/:ws_id/shifts/assign",
            post(handlers::assign_shift),
        )
        .layer(cors)
        .with_state(state)
}
