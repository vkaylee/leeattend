use axum::{routing::get, Router};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/dashboard/executive-overview",
            get(handlers::get_dashboard_metrics),
        )
        .route(
            "/dashboard/overtime-breakdown",
            get(handlers::dashboard_overtime_breakdown::get_overtime_breakdown),
        )
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
        .route("/dashboard/peak-times", get(handlers::get_peak_times))
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
}
