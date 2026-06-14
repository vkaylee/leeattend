use crate::handlers;
use crate::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Folders
        .route(
            "/workspaces/:ws_id/documents/folders",
            get(handlers::list_folders_handler).post(handlers::create_folder_handler),
        )
        .route(
            "/workspaces/:ws_id/documents/folders/:id",
            get(handlers::list_folders_handler) // Stub get
                .put(handlers::rename_folder_handler)
                .delete(handlers::delete_folder_handler),
        )
        // Distributions
        .route(
            "/workspaces/:ws_id/documents/distributions",
            post(handlers::distribute_document_handler),
        )
        // ESS (Employee Self-Service)
        .route(
            "/ess/documents/pending",
            get(handlers::get_pending_documents_handler),
        )
        .route(
            "/ess/documents/:id/acknowledge",
            post(handlers::acknowledge_document_handler),
        )
        // Templates
        .route(
            "/workspaces/:ws_id/documents/templates/preview",
            post(handlers::preview_template_handler),
        )
        .route(
            "/workspaces/:ws_id/documents/templates/:id/generate",
            post(handlers::generate_document_from_template_handler),
        )
}
