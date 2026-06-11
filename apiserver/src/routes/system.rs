use axum::{
    routing::{delete, get, patch},
    Router,
};
use std::sync::Arc;

use crate::handlers::system_admin;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // System role management (PlatformAdmin only)
        .route("/system/roles", get(system_admin::list_system_roles))
        .route(
            "/system/users/:user_id/roles",
            get(system_admin::get_user_system_roles).post(system_admin::assign_system_role),
        )
        .route(
            "/system/users/:user_id/roles/:role_name",
            delete(system_admin::revoke_system_role),
        )
        // Infrastructure health (PlatformAdmin + Support)
        .route("/system/health", get(system_admin::system_health))
        // System DB Clusters
        .route(
            "/system/db-clusters",
            get(crate::handlers::system_db_clusters::list_db_clusters)
                .post(crate::handlers::system_db_clusters::create_db_cluster),
        )
        .route(
            "/system/db-clusters/:id",
            delete(crate::handlers::system_db_clusters::delete_db_cluster),
        )
        .route(
            "/system/db-clusters/:id/status",
            patch(crate::handlers::system_db_clusters::update_db_cluster_status),
        )
}
