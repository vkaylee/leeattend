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
pub mod services;
pub mod utils;
pub mod worker;
pub mod mw;

use axum::{
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};

use crate::agent_manager::AgentManager;
use crate::db_pool::DbPools;
use crate::queue::RedisPool;
use crate::repositories::{AgentRepository, EmployeeRepository, WorkspaceRepository};
use crate::repositories::audit_log_repository::AuditLogRepository;
use crate::services::{AgentService, AttendanceLogService, AttendanceSummaryService, EmployeeService, ReportCoordinator, ReportService, TimeTrackingService};
use crate::services::attendance_adjustment_service::AttendanceAdjustmentService;
use crate::services::attendance_import_service::AttendanceImportService;
use crate::services::auth_service::AuthService;
use crate::services::holiday_leave_service::HolidayLeaveService;
use crate::services::leave_accrual_service::LeaveAccrualService;
use crate::services::leave_service::LeaveService;
use crate::services::payroll_service::PayrollService;
use crate::services::pairing_service::PairingService;
use crate::services::shift_service::ShiftService;
use crate::services::shift_swap_service::ShiftSwapService;
use crate::services::workspace_service::WorkspaceService;
use axum::http::{HeaderValue, Method};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub struct AppState {
    pub agent_manager: AgentManager,
    pub db: DbPools,
    pub agent_repo: Arc<dyn AgentRepository>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub audit_repo: Arc<dyn AuditLogRepository>,
    pub employee_repo: Arc<dyn EmployeeRepository>,
    pub agent_service: Arc<AgentService>,
    pub employee_service: Arc<EmployeeService>,
    pub time_tracking_service: Arc<TimeTrackingService>,
    pub attendance_summary_service: Arc<AttendanceSummaryService>,
    pub report_service: Arc<ReportService>,
    pub pairing_service: Arc<PairingService>,
    pub adjustment_service: Arc<AttendanceAdjustmentService>,
    pub import_service: Arc<AttendanceImportService>,
    pub payroll_service: Arc<PayrollService>,
    pub auth_service: Arc<AuthService>,
    pub shift_service: Arc<ShiftService>,
    pub workspace_service: Arc<WorkspaceService>,
    pub leave_service: Arc<LeaveService>,
    pub shift_swap_service: Arc<ShiftSwapService>,
    pub holiday_leave_service: Arc<HolidayLeaveService>,
    pub leave_accrual_service: Arc<LeaveAccrualService>,
    pub jwt_secret: String,
    pub attendance_service: Arc<AttendanceLogService>,

    /// Optional Redis pool for distributed job processing.
    /// When None, report generation falls back to synchronous processing.
    pub redis: Option<RedisPool>,
    /// Optional ReportCoordinator for async report generation.
    /// Requires Redis to be available.
    pub coordinator: Option<Arc<ReportCoordinator>>,
    /// Optional roster cache (requires Redis). Falls back to direct DB on None.
    pub roster_cache: Option<Arc<crate::cache::RosterCache>>,
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
    let cors = if cfg!(debug_assertions) {
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
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderName::from_static("x-requested-with"),
            ])
    };

    let mut router = Router::new()
        .route(
            "/dashboard/workspace-stats",
            get(handlers::get_workspace_stats),
        )
        .route(
            "/dashboard/attendance-trend",
            get(handlers::get_attendance_trend),
        )
        .route(
            "/dashboard/department-attendance",
            get(handlers::get_department_attendance),
        )
        .route(
            "/dashboard/unexcused-absences",
            get(handlers::workspaces::crud::get_unexcused_absences),
        )
        .route(
            "/dashboard/overtime-trend",
            get(handlers::get_overtime_trend),
        )
        .route(
            "/dashboard/peak-times",
            get(handlers::get_peak_times),
        )
        .route(
            "/dashboard/leave-types",
            get(handlers::get_leave_types_breakdown),
        )
        .route(
            "/dashboard/lateness-by-day",
            get(handlers::get_lateness_by_day),
        )
        .route(
            "/dashboard/recruitment-sources",
            get(handlers::recruitment_sources::get_recruitment_source_breakdown),
        )
        .route(
            "/dashboard/turnover-analytics",
            get(handlers::recruitment_sources::get_turnover_analytics),
        )
        .route(
            "/dashboard/preferences",
            get(handlers::dashboard_preferences::get_dashboard_preferences)
                .put(handlers::dashboard_preferences::update_dashboard_preferences),
        )
        .route("/auth/login", post(handlers::login))
        .route("/auth/register", post(handlers::register))
        .route_layer(mw::RateLimitLayer::new(
            state.redis.clone(),
            mw::rate_limit::RateLimitConfig::default(),
            state.jwt_secret.clone(),
        ))
        .route("/health", get(handlers::health_check))
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
        tracing::warn!("⚠️ DEV MODE ACTIVE: Debug auth bypass & data-clearing endpoints are enabled. Never ship this to production!");
    }

    router = router
        .route("/ws", get(handlers::ws_handler))
        .route("/agents", get(handlers::get_agents))
        .route("/agent/:agent_id/command", post(handlers::send_command))
        .route("/me", get(handlers::get_me).patch(handlers::update_me))
        .route("/me/permissions", get(handlers::get_my_permissions))
        .route("/auth/change-password", post(handlers::change_password))
        .route("/users", get(handlers::get_users))
        .route(
            "/devices",
            get(handlers::get_devices).post(handlers::create_device),
        )
        .route(
            "/devices/:id",
            put(handlers::update_device).delete(handlers::delete_device),
        )
        .route(
            "/devices/:id/info",
            get(handlers::get_device_info_on_demand),
        )
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
        .route(
            "/workspaces/:ws_id/members/:user_id/permissions",
            get(handlers::get_member_extra_permissions).post(handlers::assign_extra_permission),
        )
        .route(
            "/workspaces/:ws_id/members/:user_id/permissions/:perm_id",
            delete(handlers::revoke_extra_permission),
        )
        .route(
            "/workspaces/:ws_id/permissions/available",
            get(handlers::get_available_permissions),
        )
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
            get(handlers::leave_types::list_leave_types)
                .post(handlers::leave_types::create_leave_type),
        )
        .route(
            "/workspaces/:ws_id/leave/types/:id",
            delete(handlers::leave_types::delete_leave_type),
        )
        .route(
            "/workspaces/:ws_id/leave/types/:id/config",
            put(handlers::leave_types::update_leave_type_config),
        )
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
            "/workspaces/:ws_id/leave/policies/assignments",
            get(handlers::leaves::get_leave_policy_assignments),
        )
        .route(
            "/workspaces/:ws_id/leave/accrual/run",
            post(handlers::trigger_accrual_run),
        )
        .route(
            "/workspaces/:ws_id/jobs/:job_id/status",
            get(handlers::leaves::get_job_status),
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
            "/workspaces/:ws_id/attendance/staging-logs",
            get(handlers::attendance::get_staging_logs),
        )
        .route(
            "/workspaces/:ws_id/attendance/staging-logs/:id/resolve",
            post(handlers::attendance::resolve_staging_log),
        )
        .route(
            "/workspaces/:ws_id/attendance/staging-logs/:id",
            delete(handlers::attendance::dismiss_staging_log),
        )
        .route(
            "/workspaces/:ws_id/attendance/claim-ot/:employee_id",
            post(handlers::claim_ot),
        )
        .route(
            "/workspaces/:ws_id/attendance/enter-paper-ot/:employee_id",
            post(handlers::enter_paper_ot),
        )
        .route(
            "/workspaces/:ws_id/attendance/bulk-upload-paper-ot",
            post(handlers::bulk_upload_paper_ot),
        )
        .route(
            "/workspaces/:ws_id/attendance/approve-ot/:employee_id",
            post(handlers::approve_ot),
        )
        .route(
            "/workspaces/:ws_id/attendance/summaries",
            get(handlers::get_attendance_summaries),
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
        // Async report export endpoints
        .route(
            "/workspaces/:ws_id/reports/export",
            post(handlers::report_export::create_export_job),
        )
        .route(
            "/workspaces/:ws_id/reports/jobs/:job_id",
            get(handlers::report_export::get_job_status)
                .delete(handlers::report_export::cancel_job),
        )
        .route(
            "/workspaces/:ws_id/reports/jobs/:job_id/download",
            get(handlers::report_export::download_report),
        )
        // WebSocket endpoint for real-time report streaming
        // Auth middleware runs before WebSocketUpgrade extractor to return 401 properly
        .route(
            "/ws/reports/:job_id",
            get(handlers::report_websocket::report_stream_ws).layer(
                middleware::from_fn_with_state(
                    state.clone(),
                    handlers::report_websocket::ws_auth_middleware,
                ),
            ),
        )
        // Worker monitoring endpoint (global infrastructure, single combined call)
        .route(
            "/monitor/overview",
            get(handlers::worker_monitor::get_monitor_overview),
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
        .route(
            "/workspaces/:ws_id/holidays",
            get(handlers::holidays::list_holidays)
                .post(handlers::holidays::create_holiday),
        )
        .route(
            "/workspaces/:ws_id/holidays/:id",
            get(handlers::holidays::get_holiday)
                .put(handlers::holidays::update_holiday)
                .patch(handlers::holidays::update_holiday)
                .delete(handlers::holidays::delete_holiday),
        )
        .route(
            "/workspaces/:ws_id/holidays/:id/process-leave",
            post(handlers::holidays::process_holiday_leave),
        )
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
            post(handlers::set_department_payroll_config)
                .get(handlers::get_department_payroll_configs),
        )
        .route(
            "/workspaces/:ws_id/recruitment-sources",
            get(handlers::recruitment_sources::get_recruitment_sources)
                .post(handlers::recruitment_sources::create_recruitment_source),
        )
        .route(
            "/workspaces/:ws_id/recruitment-sources/:id",
            put(handlers::recruitment_sources::update_recruitment_source)
                .delete(handlers::recruitment_sources::delete_recruitment_source),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/departments",
            get(handlers::get_employee_department_history).post(handlers::transfer_employee),
        )
        .route(
            "/workspaces/:ws_id/audit-logs",
            get(handlers::workspaces::audit::get_audit_logs),
        )
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
        .route(
            "/workspaces/:ws_id/assignment-rules",
            get(handlers::assignment_rules::get_assignment_rules)
                .post(handlers::assignment_rules::create_assignment_rule),
        )
        .route(
            "/workspaces/:ws_id/assignment-rules/:rule_id",
            patch(handlers::assignment_rules::update_assignment_rule)
                .delete(handlers::assignment_rules::delete_assignment_rule),
        )
        // Shift swap routes
        .route(
            "/workspaces/:ws_id/shift-swaps",
            get(handlers::shift_swaps::list_shift_swaps)
                .post(handlers::shift_swaps::create_shift_swap),
        )
        .route(
            "/workspaces/:ws_id/shift-swaps/stats",
            get(handlers::shift_swaps::get_shift_swap_stats),
        )
        .route(
            "/workspaces/:ws_id/shift-swaps/:id/approve",
            put(handlers::shift_swaps::approve_shift_swap),
        );

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

    let ess_routes = Router::new()
        .route(
            "/ess/profile",
            get(handlers::ess::get_my_profile).put(handlers::ess::update_my_profile),
        )
        .route(
            "/ess/emergency-contacts",
            get(handlers::ess::get_my_emergency_contacts)
                .post(handlers::ess::add_emergency_contact),
        )
        .route(
            "/ess/emergency-contacts/:contact_id",
            delete(handlers::ess::delete_emergency_contact),
        )
        .route("/ess/documents", get(handlers::ess::get_my_documents))
        .route("/ess/payslips", get(handlers::ess::get_my_payslips))
        .route("/ess/attendance", get(handlers::ess::get_my_attendance))
        .route(
            "/ess/leave/balances",
            get(handlers::ess::get_my_leave_balances),
        )
        .route(
            "/ess/leave/requests",
            get(handlers::ess::ess_get_my_leave_requests),
        )
        // ESS Shift Swap routes (Phase 2)
        .route(
            "/ess/shift-swaps",
            get(handlers::ess::ess_get_my_shift_swaps)
                .post(handlers::ess::ess_create_shift_swap),
        )
        .route(
            "/ess/shift-swaps/:id/respond",
            put(handlers::ess::ess_respond_to_swap),
        )
        .route(
            "/ess/shift-swaps/:id",
            delete(handlers::ess::ess_cancel_swap),
        );

    Router::new()
        .nest("/api", router)
        .nest("/api", roster_routes)
        .nest("/api", ess_routes)
        .layer(cors)
        .with_state(state)
}
