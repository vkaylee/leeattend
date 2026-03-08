use axum::{middleware, routing::{get, post}, Router};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn router(state: &Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
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
        .route(
            "/ws/reports/:job_id",
            get(handlers::report_websocket::report_stream_ws).layer(
                middleware::from_fn_with_state(
                    state.clone(),
                    handlers::report_websocket::ws_auth_middleware,
                ),
            ),
        )
        // Worker monitoring endpoint
        .route(
            "/monitor/overview",
            get(handlers::worker_monitor::get_monitor_overview),
        )
}
