use axum::{
    routing::{delete, get, patch},
    Router,
};
use std::sync::Arc;

use crate::handlers::system_admin;
use crate::handlers::system_tenants;
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
        .route(
            "/system/tenants",
            get(system_tenants::list_tenants).post(system_tenants::create_tenant),
        )
        .route(
            "/system/tenants/stale",
            get(system_tenants::list_stale_tenants),
        )
        .route(
            "/system/tenants/:tenant_id",
            delete(system_tenants::delete_tenant_handler),
        )
        .route(
            "/system/tenants/:tenant_id/stats",
            get(system_tenants::get_tenant_stats),
        )
        .route(
            "/system/tenants/:tenant_id/database",
            axum::routing::put(system_tenants::update_tenant_database),
        )
        .route(
            "/system/tenants/:tenant_id/mode",
            patch(system_tenants::update_tenant_mode),
        )
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
