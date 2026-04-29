use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
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
}
