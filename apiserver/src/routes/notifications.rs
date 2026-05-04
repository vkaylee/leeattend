use crate::handlers::notifications::{
    get_notifications, get_unread_count, mark_all_as_read, mark_as_read,
};
use crate::AppState;
use axum::{
    routing::{get, patch, post},
    Router,
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/notifications", get(get_notifications))
        .route("/v1/notifications/unread-count", get(get_unread_count))
        .route("/v1/notifications/:id/read", patch(mark_as_read))
        .route("/v1/notifications/read-all", post(mark_all_as_read))
}
