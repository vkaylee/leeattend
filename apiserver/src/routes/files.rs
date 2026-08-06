use crate::handlers;
use crate::utils::env_registry::EnvRegistry;
use crate::AppState;
use axum::extract::DefaultBodyLimit;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn router(registry: &EnvRegistry) -> Router<Arc<AppState>> {
    let max_upload_bytes = registry
        .get_parsed::<usize>("MAX_FILE_UPLOAD_BODY_BYTES")
        .unwrap_or(20 * 1024 * 1024);

    Router::new()
        .route(
            "/workspaces/:ws_id/files/upload",
            post(handlers::upload_file_handler),
        )
        .route(
            "/workspaces/:ws_id/files/:id",
            get(handlers::get_file_metadata_handler)
                .put(handlers::update_file_handler)
                .delete(handlers::delete_file_handler),
        )
        .route_layer(DefaultBodyLimit::max(max_upload_bytes))
        .route(
            "/workspaces/:ws_id/files/presigned",
            post(handlers::create_upload_url_handler),
        )
        .route(
            "/workspaces/:ws_id/files/:id/download",
            get(handlers::download_file_handler),
        )
        .route(
            "/workspaces/:ws_id/files/:id/url",
            get(handlers::get_file_url_handler),
        )
        .route(
            "/workspaces/:ws_id/files/:id/versions",
            get(handlers::list_file_versions_handler),
        )
        .route(
            "/workspaces/:ws_id/files/by-owner/:owner_type/:owner_id",
            get(handlers::list_files_by_owner_handler),
        )
}
