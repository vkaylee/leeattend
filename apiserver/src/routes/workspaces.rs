use axum::{routing::{delete, get, patch, post, put}, Router};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // User/Me routes
        .route("/me", get(handlers::get_me).patch(handlers::update_me))
        .route("/me/permissions", get(handlers::get_my_permissions))
        .route("/users", get(handlers::get_users))
        // WebSocket
        .route("/ws", get(handlers::ws_handler))
        // Agents & Devices
        .route("/agents", get(handlers::get_agents))
        .route("/agent/:agent_id/command", post(handlers::send_command))
        .route(
            "/devices",
            get(handlers::get_devices).post(handlers::create_device),
        )
        .route(
            "/devices/:id",
            put(handlers::update_device).delete(handlers::delete_device),
        )
        .route(
            "/devices/:id/info",
            get(handlers::get_device_info_on_demand),
        )
        .route(
            "/authorized-agents",
            get(handlers::get_authorized_agents).post(handlers::create_authorized_agent),
        )
        .route(
            "/authorized-agents/:id",
            get(handlers::get_authorized_agent_by_id)
                .patch(handlers::update_authorized_agent)
                .delete(handlers::delete_authorized_agent),
        )
        .route(
            "/authorized-agents/:id/trust",
            put(handlers::update_agent_trust),
        )
        // Workspaces CRUD & members
        .route(
            "/workspaces",
            get(handlers::get_workspaces).post(handlers::create_workspace),
        )
        .route(
            "/workspaces/:ws_id",
            get(handlers::get_workspace_by_id).patch(handlers::update_workspace),
        )
        .route(
            "/workspaces/:ws_id/members",
            get(handlers::get_workspace_members).post(handlers::add_workspace_member),
        )
        .route(
            "/workspaces/:ws_id/members/:user_id",
            patch(handlers::update_workspace_member_role).delete(handlers::remove_workspace_member),
        )
        .route(
            "/workspaces/:ws_id/members/:user_id/roles",
            get(handlers::get_member_roles).post(handlers::assign_extra_role),
        )
        .route(
            "/workspaces/:ws_id/members/:user_id/roles/:role_id",
            delete(handlers::revoke_extra_role),
        )
        .route(
            "/workspaces/:ws_id/members/:user_id/permissions",
            get(handlers::get_member_extra_permissions).post(handlers::assign_extra_permission),
        )
        .route(
            "/workspaces/:ws_id/members/:user_id/permissions/:perm_id",
            delete(handlers::revoke_extra_permission),
        )
        .route(
            "/workspaces/:ws_id/permissions/available",
            get(handlers::get_available_permissions),
        )
        // Audit logs
        .route(
            "/workspaces/:ws_id/audit-logs",
            get(handlers::workspaces::audit::get_audit_logs),
        )
        // Jobs
        .route(
            "/workspaces/:ws_id/jobs/:job_id",
            get(handlers::get_generic_job_status),
        )
}
