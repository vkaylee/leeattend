use axum::{routing::get, Router};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Dashboard task feed
        .route(
            "/workspaces/:ws_id/lifecycle/my-tasks",
            get(handlers::lifecycle_checklists::get_my_tasks),
        )
        // Templates CRUD
        .route(
            "/workspaces/:ws_id/lifecycle-templates",
            get(handlers::lifecycle_checklists::list_templates)
                .post(handlers::lifecycle_checklists::create_template),
        )
        .route(
            "/workspaces/:ws_id/lifecycle-templates/:id",
            get(handlers::lifecycle_checklists::get_template)
                .patch(handlers::lifecycle_checklists::update_template)
                .delete(handlers::lifecycle_checklists::delete_template),
        )
        // Template Items CRUD
        .route(
            "/workspaces/:ws_id/lifecycle-templates/:id/items",
            get(handlers::lifecycle_checklists::list_template_items)
                .post(handlers::lifecycle_checklists::create_template_item),
        )
        .route(
            "/workspaces/:ws_id/lifecycle-templates/:tid/items/:iid",
            get(handlers::lifecycle_checklists::get_template_item)
                .put(handlers::lifecycle_checklists::update_template_item)
                .delete(handlers::lifecycle_checklists::delete_template_item),
        )
        // Checklist Instances
        .route(
            "/workspaces/:ws_id/employments/:emp_id/lifecycle",
            get(handlers::lifecycle_checklists::list_checklists)
                .post(handlers::lifecycle_checklists::create_checklist),
        )
        .route(
            "/workspaces/:ws_id/lifecycle/:id",
            get(handlers::lifecycle_checklists::get_checklist),
        )
        .route(
            "/workspaces/:ws_id/lifecycle/:id/items/:iid",
            // Use patch since we are updating partial fields (status/notes) of a single checklist item
            axum::routing::patch(handlers::lifecycle_checklists::update_item_status),
        )
}
