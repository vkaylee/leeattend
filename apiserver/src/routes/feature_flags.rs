use axum::{routing::get, Router};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/workspaces/:ws_id/feature-flags",
            get(handlers::feature_flags::list_feature_flags)
                .post(handlers::feature_flags::create_feature_flag)
                .put(handlers::feature_flags::upsert_feature_flag),
        )
        .route(
            "/workspaces/:ws_id/feature-flags/:id",
            get(handlers::feature_flags::get_feature_flag)
                .patch(handlers::feature_flags::update_feature_flag)
                .delete(handlers::feature_flags::delete_feature_flag),
        )
        .route(
            "/workspaces/:ws_id/feature-flags/check/:key",
            get(handlers::feature_flags::check_feature_flag),
        )
        // ADR-005: Cascade dual-run window status
        .route(
            "/workspaces/:ws_id/cascade-dual-run/status",
            get(handlers::feature_flags::get_cascade_dual_run_status),
        )
}
