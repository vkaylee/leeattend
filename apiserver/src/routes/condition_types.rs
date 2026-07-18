use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::handlers::condition_types;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/workspaces/:ws_id/condition-types",
            get(condition_types::list_condition_types).post(condition_types::create_condition_type),
        )
        .route(
            "/workspaces/:ws_id/condition-types/:id",
            get(condition_types::get_condition_type)
                .patch(condition_types::update_condition_type)
                .delete(condition_types::delete_condition_type),
        )
}
