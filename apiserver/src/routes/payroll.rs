use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/workspaces/:ws_id/payroll-settings",
            get(handlers::get_payroll_settings).patch(handlers::update_payroll_settings),
        )
        .route(
            "/workspaces/:ws_id/payroll-settings/history",
            get(handlers::get_payroll_settings_history),
        )
        .route(
            "/workspaces/:ws_id/salary-components",
            get(handlers::get_salary_components).post(handlers::create_salary_component),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/salary-components",
            get(handlers::get_employee_salary_components)
                .post(handlers::assign_employee_salary_component),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/payslips",
            get(handlers::get_employee_payslips),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/calculate-payroll",
            post(handlers::calculate_payroll),
        )
        .route(
            "/workspaces/:ws_id/payroll/calculate-batch",
            post(handlers::calculate_batch_payroll),
        )
        .route(
            "/workspaces/:ws_id/payroll/payslips",
            get(handlers::get_workspace_payslips),
        )
        .route(
            "/workspaces/:ws_id/payroll/incentive-policies",
            get(handlers::get_incentive_policies).post(handlers::create_incentive_policy),
        )
        .route(
            "/workspaces/:ws_id/payroll/incentive-policies/:policy_id",
            delete(handlers::delete_incentive_policy),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/payroll-config",
            post(handlers::set_employee_payroll_config).get(handlers::get_employee_payroll_configs),
        )
        // Overtime policies
        .route(
            "/workspaces/:ws_id/overtime/policies",
            get(handlers::list_policies).post(handlers::create_policy),
        )
        .route(
            "/workspaces/:ws_id/overtime/policies/:policy_id",
            get(handlers::get_policy),
        )
        .route(
            "/workspaces/:ws_id/overtime/policies/:policy_id/rules",
            post(handlers::add_rule),
        )
        .route(
            "/workspaces/:ws_id/overtime/policies/assign",
            post(handlers::assign_policy),
        )
}
