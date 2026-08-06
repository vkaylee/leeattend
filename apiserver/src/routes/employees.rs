use axum::extract::DefaultBodyLimit;
use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::utils::env_registry::EnvRegistry;
use crate::AppState;

#[allow(clippy::too_many_lines)]
pub fn router(registry: &EnvRegistry) -> Router<Arc<AppState>> {
    let max_bulk_upload_bytes = registry
        .get_parsed::<usize>("MAX_BULK_UPLOAD_BODY_BYTES")
        .unwrap_or(10 * 1024 * 1024);

    Router::new()
        .route(
            "/workspaces/:ws_id/employees/upload",
            post(handlers::upload_employee_excel),
        )
        .route_layer(DefaultBodyLimit::max(max_bulk_upload_bytes))
        .route(
            "/workspaces/:ws_id/employees/import-termination/preview",
            post(handlers::import_termination_preview),
        )
        .route(
            "/workspaces/:ws_id/employees/import-termination/confirm",
            post(handlers::import_termination_confirm),
        )
        .route(
            "/workspaces/:ws_id/employees/import-termination/template",
            get(handlers::import_termination_template),
        )
        .route(
            "/workspaces/:ws_id/employees",
            get(handlers::get_employees).post(handlers::create_employee),
        )
        .route(
            "/workspaces/:ws_id/employees/:id",
            get(handlers::get_employee)
                .put(handlers::update_employee)
                .delete(handlers::delete_employee),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/rehire",
            post(handlers::rehire_employee),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/conditions",
            get(handlers::get_working_conditions).post(handlers::create_working_condition),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/conditions/:cond_id",
            delete(handlers::delete_working_condition),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/departments",
            get(handlers::get_employee_department_history).post(handlers::transfer_employee),
        )
        .route(
            "/workspaces/:ws_id/employees/:id/compensation-history",
            get(handlers::get_compensation_history),
        )
        // Employee Contracts (v1 prefix as requested)
        .route(
            "/workspaces/:ws_id/contracts",
            get(handlers::employee_contracts::list_workspace_contracts),
        )
        .route(
            "/workspaces/:ws_id/employees/:employee_id/contracts",
            get(handlers::employee_contracts::list_employee_contracts)
                .post(handlers::employee_contracts::create_contract),
        )
        .route(
            "/workspaces/:ws_id/employees/:employee_id/contracts/:contract_id",
            get(handlers::employee_contracts::get_contract)
                .patch(handlers::employee_contracts::update_contract)
                .delete(handlers::employee_contracts::delete_contract),
        )
        // Employee Employments
        .route(
            "/workspaces/:ws_id/employees/:employee_id/employments",
            get(handlers::employments::list_employee_employments)
                .post(handlers::employments::create_employment),
        )
        .route(
            "/workspaces/:ws_id/employees/:employee_id/employments/:employment_id",
            get(handlers::employments::get_employment)
                .patch(handlers::employments::update_employment)
                .delete(handlers::employments::delete_employment),
        )
        // Employee Assignments (Secondment)
        .route(
            "/workspaces/:ws_id/employees/:employee_id/assignments",
            get(handlers::employee_assignments::list_employee_assignments)
                .post(handlers::employee_assignments::create_employee_assignment),
        )
        .route(
            "/workspaces/:ws_id/employees/:employee_id/assignments/:assignment_id",
            get(handlers::employee_assignments::get_employee_assignment)
                .patch(handlers::employee_assignments::update_employee_assignment)
                .delete(handlers::employee_assignments::delete_employee_assignment),
        )
        // Recruitment sources
        .route(
            "/workspaces/:ws_id/recruitment-sources",
            get(handlers::recruitment_sources::get_recruitment_sources)
                .post(handlers::recruitment_sources::create_recruitment_source),
        )
        .route(
            "/workspaces/:ws_id/recruitment-sources/:id",
            put(handlers::recruitment_sources::update_recruitment_source)
                .delete(handlers::recruitment_sources::delete_recruitment_source),
        )
        // Departments
        .route(
            "/workspaces/:ws_id/departments",
            get(handlers::get_departments).post(handlers::create_department),
        )
        .route(
            "/workspaces/:ws_id/departments/:id",
            patch(handlers::update_department).delete(handlers::delete_department),
        )
        .route(
            "/workspaces/:ws_id/departments/:id/payroll-config",
            post(handlers::set_department_payroll_config)
                .get(handlers::get_department_payroll_configs),
        )
}
