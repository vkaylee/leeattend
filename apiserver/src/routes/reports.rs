use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn router(state: &Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/workspaces/:ws_id/reports/attendance",
            get(handlers::get_attendance_report),
        )
        .route(
            "/workspaces/:ws_id/reports/no-show",
            get(handlers::get_no_show_report),
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
        .route(
            "/ws/reports/:job_id",
            get(handlers::report_websocket::report_stream_ws).layer(
                middleware::from_fn_with_state(
                    Arc::clone(state),
                    handlers::report_websocket::ws_auth_middleware,
                ),
            ),
        )
        // Worker monitoring endpoint
        .route(
            "/monitor/overview",
            get(handlers::worker_monitor::get_monitor_overview),
        )
        // Worker monitoring jobs paginated endpoint
        .route(
            "/monitor/jobs",
            get(handlers::worker_monitor::get_monitor_jobs),
        )
        // Safe durable job correlation lookup
        .route(
            "/monitor/correlations",
            get(handlers::worker_monitor::get_monitor_correlations),
        )
        // Worker monitoring SSE stream
        .route(
            "/monitor/stream",
            get(handlers::worker_monitor::monitor_stream_sse),
        )
        .route(
            "/monitor/alerts",
            get(handlers::monitor_alerts::list_monitor_alerts),
        )
        .route(
            "/monitor/alerts/:alert_id",
            get(handlers::monitor_alerts::get_monitor_alert),
        )
        .route(
            "/monitor/alerts/:alert_id/acknowledge",
            post(handlers::monitor_alerts::acknowledge_monitor_alert),
        )
        .route(
            "/monitor/alerts/:alert_id/resolve",
            post(handlers::monitor_alerts::resolve_monitor_alert),
        )
}
