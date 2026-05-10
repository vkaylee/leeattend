use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/workspaces/:ws_id/employees/upload",
            post(handlers::upload_employee_excel),
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
        // Employee Contracts (v1 prefix as requested)
        .route(
            "/v1/workspaces/:ws_id/employees/:employee_id/contracts",
            get(handlers::employee_contracts::list_employee_contracts)
                .post(handlers::employee_contracts::create_contract),
        )
        .route(
            "/v1/workspaces/:ws_id/employees/:employee_id/contracts/:contract_id",
            get(handlers::employee_contracts::get_contract)
                .patch(handlers::employee_contracts::update_contract)
                .delete(handlers::employee_contracts::delete_contract),
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
