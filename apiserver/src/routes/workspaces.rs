use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

#[allow(clippy::too_many_lines)]
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
            "/workspaces/:ws_id/roles",
            get(handlers::get_workspace_roles).post(handlers::create_workspace_role),
        )
        .route(
            "/workspaces/:ws_id/roles/:role_id",
            put(handlers::update_workspace_role).delete(handlers::delete_workspace_role),
        )
        .route(
            "/workspaces/:ws_id/roles/:role_id/permissions",
            get(handlers::get_role_permissions).put(handlers::update_role_permissions),
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
        // Employment Phases
        .route(
            "/workspaces/:ws_id/employment-phases",
            get(handlers::employment_phases::get_employment_phase_definitions)
                .post(handlers::employment_phases::create_employment_phase_definition),
        )
        .route(
            "/workspaces/:ws_id/employees/:user_id/phases",
            get(handlers::employment_phases::get_employee_phases)
                .post(handlers::employment_phases::assign_employee_phase),
        )
        .route(
            "/workspaces/:ws_id/employees/:user_id/phases/:phase_id/status",
            put(handlers::employment_phases::update_employee_phase_status),
        )
        // Payroll Rules
        .route(
            "/workspaces/:ws_id/payroll-rules",
            get(handlers::payroll_rules::get_payroll_rule_definitions)
                .post(handlers::payroll_rules::create_payroll_rule_definition),
        )
        .route(
            "/workspaces/:ws_id/payroll-rules/:rule_id/tiers",
            get(handlers::payroll_rules::get_payroll_rule_tiers)
                .post(handlers::payroll_rules::create_payroll_rule_tier),
        )
        .route(
            "/workspaces/:ws_id/payroll-rules/:rule_id/tiers/:tier_id",
            delete(handlers::payroll_rules::delete_payroll_rule_tier),
        )
        .route(
            "/workspaces/:ws_id/payroll-rules/:rule_id/assignments",
            get(handlers::payroll_rules::get_payroll_rule_assignments)
                .post(handlers::payroll_rules::assign_payroll_rule),
        )
        .route(
            "/workspaces/:ws_id/payroll-rules/:rule_id/assignments/:assignment_id",
            delete(handlers::payroll_rules::delete_payroll_rule_assignment),
        )
        // Jobs
        .route(
            "/workspaces/:ws_id/jobs/:job_id",
            get(handlers::get_generic_job_status),
        )
        // Salary Bases
        .route(
            "/workspaces/:ws_id/salary-bases",
            get(handlers::salary_bases::get_salary_bases)
                .post(handlers::salary_bases::create_salary_base),
        )
        .route(
            "/workspaces/:ws_id/salary-bases/:base_id/members",
            post(handlers::salary_bases::assign_component_to_base),
        )
        .route(
            "/workspaces/:ws_id/salary-bases/:base_id/members/:component_id",
            delete(handlers::salary_bases::unassign_component_from_base),
        )
        // Workspace Configs (EAV)
        .route(
            "/workspaces/:ws_id/configs",
            get(handlers::workspace_config::list_configs)
                .put(handlers::workspace_config::upsert_config),
        )
        .route(
            "/workspaces/:ws_id/configs/:key",
            get(handlers::workspace_config::get_config)
                .delete(handlers::workspace_config::delete_config),
        )
        // Approval Workflows
        .route(
            "/workspaces/:ws_id/approval-workflows",
            get(handlers::approval_workflow::list_workflows)
                .post(handlers::approval_workflow::create_workflow),
        )
        .route(
            "/workspaces/:ws_id/approval-workflows/:id",
            get(handlers::approval_workflow::get_workflow)
                .put(handlers::approval_workflow::update_workflow)
                .delete(handlers::approval_workflow::delete_workflow),
        )
        .route(
            "/workspaces/:ws_id/approval-workflows/approve",
            post(handlers::approval_workflow::process_approval),
        )
        .route(
            "/workspaces/:ws_id/approval-workflows/history/:type/:id",
            get(handlers::approval_workflow::get_approval_history),
        )
}
