pub mod agent_manager;
pub mod dev_initialization;
pub mod domain;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;
pub mod utils;

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
            // 1. In development, allow Impersonation via Header
            #[cfg(debug_assertions)]
            {
                if let Some(imp_id_str) = parts.headers.get("X-Debug-Impersonate-Id").and_then(|h| h.to_str().ok()) {
                    if let Ok(user_id) = Uuid::parse_str(imp_id_str) {
                        return Ok(UserId(user_id));
                    }
                }
            }

            // 2. Check for JWT in Authorization header
            if let Some(auth_header) = parts.headers.get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
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

            // 3. Fallback to X-User-Id in development (debug assertions)
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
        .route(
            "/dashboard/workspace-stats",
            get(handlers::get_workspace_stats),
        )
        .route("/auth/login", post(handlers::login))
        .route("/auth/register", post(handlers::register))
        .route("/auth/refresh", post(handlers::refresh_token))
        .route("/auth/logout", post(handlers::logout));

    #[cfg(debug_assertions)]
    {
        router = router.route("/dev/super-user", get(handlers::get_dev_super_user));
        router = router.route("/debug/users", get(handlers::debug::get_debug_users));
        router = router.route(
            "/debug/workspaces/:ws_id/clear",
            post(handlers::debug::clear_debug_db),
        );
        router = router.route(
            "/dev/workspaces/:ws_id/clear",
            post(handlers::clear_workspace_data),
        );
    }

    router = router
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
            "/workspaces/:ws_id",
            get(handlers::get_workspace_by_id).patch(handlers::update_workspace),
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
            "/workspaces/:ws_id/employees/upload",
            post(handlers::upload_employee_excel),
        )
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
        .route(
            "/workspaces/:ws_id/employees/:id/history",
            get(handlers::get_employee_history),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/rehire",
            post(handlers::rehire_employee),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/conditions",
            get(handlers::get_working_conditions).post(handlers::create_working_condition),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/conditions/:cond_id",
            delete(handlers::delete_working_condition),
        )
        // Time Tracking
        .route("/workspaces/:ws_id/time/punch-in", post(handlers::punch_in))
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
            "/workspaces/:ws_id/leave/balances",
            get(handlers::get_workspace_leave_balances),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/leave/balances",
            get(handlers::get_employee_leave_balances),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/leave/history",
            get(handlers::get_employee_leave_history),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/leave/adjust",
            post(handlers::adjust_leave_balance),
        )
                .route(
            "/workspaces/:ws_id/leave/types",
            get(handlers::leave_types::list_leave_types),
        )
        .route(
            "/workspaces/:ws_id/leave/types/:id/config",
            put(handlers::leave_types::update_leave_type_config),
        )
        // Leave Policies
        .route(
            "/workspaces/:ws_id/leave/policies",
            get(handlers::leaves::get_leave_policies).post(handlers::leaves::create_leave_policy),
        )
        .route(
            "/workspaces/:ws_id/leave/policies/:policy_id",
            patch(handlers::leaves::update_leave_policy)
                .delete(handlers::leaves::delete_leave_policy),
        )
        .route(
            "/workspaces/:ws_id/leave/policies/assign",
            post(handlers::leaves::assign_leave_policy),
        )
        .route(
            "/workspaces/:ws_id/leave/accrual/run",
            post(handlers::trigger_accrual_run),
        )
        .route(
            "/employees/:id/leave/requests",
            get(handlers::get_my_leave_requests),
        )
        .route(
            "/workspaces/:ws_id/attendance/adjustments",
            post(handlers::submit_attendance_adjustment)
                .get(handlers::get_workspace_attendance_adjustments),
        )
        .route(
            "/workspaces/:ws_id/attendance/upload",
            post(handlers::upload_attendance_excel),
        )
        .route(
            "/workspaces/:ws_id/attendance/logs/upload",
            post(handlers::upload_attendance_logs),
        )
        .route(
            "/workspaces/:ws_id/attendance/logs",
            get(handlers::get_attendance_logs),
        )
        .route(
            "/workspaces/:ws_id/attendance/raw-logs",
            get(handlers::get_raw_attendance_logs),
        )
        .route(
            "/workspaces/:ws_id/attendance/reprocess/:employee_id",
            post(handlers::force_reprocess_attendance),
        )
        .route(
            "/workspaces/:ws_id/attendance/import-jobs/latest",
            get(handlers::get_latest_import_job),
        )
        .route(
            "/workspaces/:ws_id/attendance/import-jobs/:id",
            get(handlers::get_import_job_status),
        )
        .route(
            "/workspaces/:ws_id/attendance/adjustments/:id/approve",
            post(handlers::approve_attendance_adjustment),
        )
        .route(
            "/employees/:id/attendance/adjustments",
            get(handlers::get_my_attendance_adjustments),
        )
        .route(
            "/workspaces/:ws_id/reports/attendance",
            get(handlers::get_attendance_report),
        )
                .route(
            "/workspaces/:ws_id/overtime/policies",
            get(handlers::list_policies).post(handlers::create_policy),
        )
        .route(
            "/workspaces/:ws_id/overtime/policies/:policy_id",
            get(handlers::get_policy),
        )
        .route(
            "/workspaces/:ws_id/overtime/policies/:policy_id/rules",
            post(handlers::add_rule),
        )
        .route(
            "/workspaces/:ws_id/overtime/policies/assign",
            post(handlers::assign_policy),
        )

        // Payroll
        .route(
            "/workspaces/:ws_id/payroll-settings",
            get(handlers::get_payroll_settings).patch(handlers::update_payroll_settings),
        )
        .route(
            "/workspaces/:ws_id/payroll-settings/history",
            get(handlers::get_payroll_settings_history),
        )
        .route(
            "/workspaces/:ws_id/salary-components",
            get(handlers::get_salary_components).post(handlers::create_salary_component),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/salary-components",
            get(handlers::get_employee_salary_components)
                .post(handlers::assign_employee_salary_component),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/payslips",
            get(handlers::get_employee_payslips),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/calculate-payroll",
            post(handlers::calculate_payroll),
        )
        .route(
            "/workspaces/:ws_id/payroll/calculate-batch",
            post(handlers::calculate_batch_payroll),
        )
        .route(
            "/workspaces/:ws_id/payroll/payslips",
            get(handlers::get_workspace_payslips),
        )
        .route(
            "/workspaces/:ws_id/payroll/incentive-policies",
            get(handlers::get_incentive_policies).post(handlers::create_incentive_policy),
        )
        .route(
            "/workspaces/:ws_id/payroll/incentive-policies/:policy_id",
            delete(handlers::delete_incentive_policy),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/payroll-config",
            post(handlers::set_employee_payroll_config).get(handlers::get_employee_payroll_configs),
        )
        // Departments
        .route(
            "/workspaces/:ws_id/departments",
            get(handlers::get_departments).post(handlers::create_department),
        )
        .route(
            "/workspaces/:ws_id/departments/:id",
            patch(handlers::update_department).delete(handlers::delete_department),
        )
        .route(
            "/workspaces/:ws_id/departments/:id/payroll-config",
            post(handlers::set_department_payroll_config).get(handlers::get_department_payroll_configs),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/departments",
            get(handlers::get_employee_department_history).post(handlers::transfer_employee),
        )
        // Shifts
        .route(
            "/workspaces/:ws_id/shifts",
            get(handlers::get_shifts).post(handlers::create_shift),
        )
        .route(
            "/workspaces/:ws_id/shifts/:shift_id",
            put(handlers::update_shift),
        )
        .route(
            "/workspaces/:ws_id/shifts/assign",
            post(handlers::assign_shift),
        )
        // Working Patterns
        .route(
            "/workspaces/:ws_id/working-patterns",
            get(handlers::working_patterns::list_patterns)
                .post(handlers::working_patterns::create_pattern),
        )
        .route(
            "/workspaces/:ws_id/working-patterns/bulk-assign",
            post(handlers::working_patterns::bulk_assign_pattern),
        )
        .route(
            "/workspaces/:ws_id/working-patterns/:pattern_id",
            get(handlers::working_patterns::get_pattern_details)
                .delete(handlers::working_patterns::delete_pattern),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/working-patterns",
            get(handlers::working_patterns::get_employee_pattern_history)
                .post(handlers::working_patterns::assign_pattern),
        )
        // Assignment Rules
        .route(
            "/workspaces/:ws_id/assignment-rules",
            get(handlers::assignment_rules::get_assignment_rules)
                .post(handlers::assignment_rules::create_assignment_rule),
        )
        .route(
            "/workspaces/:ws_id/assignment-rules/:rule_id",
            patch(handlers::assignment_rules::update_assignment_rule)
                .delete(handlers::assignment_rules::delete_assignment_rule),
        );

    // Rosters
    let roster_routes = Router::new()
        .route(
            "/workspaces/:ws_id/schedules",
            get(handlers::get_workspace_schedules),
        )
        .route(
            "/workspaces/:ws_id/assignment-history",
            get(handlers::get_assignment_history),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/rosters",
            get(handlers::get_employee_rosters).post(handlers::upsert_roster),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/rosters/bulk",
            post(handlers::bulk_upsert_roster),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/rosters/:date",
            delete(handlers::delete_roster),
        );

    // Employee Self-Service (ESS) Routes
    let ess_routes = Router::new()
        // Profile
        .route(
            "/ess/profile",
            get(handlers::ess::get_my_profile).put(handlers::ess::update_my_profile),
        )
        // Emergency Contacts
        .route(
            "/ess/emergency-contacts",
            get(handlers::ess::get_my_emergency_contacts)
                .post(handlers::ess::add_emergency_contact),
        )
        .route(
            "/ess/emergency-contacts/:contact_id",
            delete(handlers::ess::delete_emergency_contact),
        )
        // Documents
        .route("/ess/documents", get(handlers::ess::get_my_documents))
        // Payslips
        .route("/ess/payslips", get(handlers::ess::get_my_payslips))
        // Attendance
        .route("/ess/attendance", get(handlers::ess::get_my_attendance))
        // Leave
        .route(
            "/ess/leave/balances",
            get(handlers::ess::get_my_leave_balances),
        )
        .route(
            "/ess/leave/requests",
            get(handlers::ess::ess_get_my_leave_requests),
        );

    Router::new()
        .nest("/api", router)
        .nest("/api", roster_routes)
        .nest("/api", ess_routes)
        .layer(cors)
        .with_state(state)
}
