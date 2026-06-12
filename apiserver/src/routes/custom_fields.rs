use axum::{routing::get, Router};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/workspaces/:ws_id/custom-fields/definitions",
            get(handlers::custom_fields::list_definitions)
                .post(handlers::custom_fields::create_definition),
        )
        .route(
            "/workspaces/:ws_id/custom-fields/definitions/:id",
            get(handlers::custom_fields::get_definition)
                .patch(handlers::custom_fields::update_definition)
                .delete(handlers::custom_fields::delete_definition),
        )
        .route(
            "/workspaces/:ws_id/custom-fields/values/:entity_type/:entity_id",
            get(handlers::custom_fields::get_values).put(handlers::custom_fields::set_values),
        )
}
