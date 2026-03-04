use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{
    AssignSalaryComponentRequest, CreateSalaryComponentRequest, EmployeeSalaryComponent,
    SalaryComponentDefinition,
};
use crate::AppState;

pub async fn get_salary_components(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> AppResult<Json<Vec<SalaryComponentDefinition>>> {
    let components = crate::repositories::payroll_repository::get_salary_component_definitions(
        state.db.api(),
        workspace_id,
    )
    .await?;

    Ok(Json(components))
}

pub async fn create_salary_component(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<CreateSalaryComponentRequest>,
) -> AppResult<(StatusCode, Json<SalaryComponentDefinition>)> {
    let component = crate::repositories::payroll_repository::create_salary_component_definition(
        state.db.api(),
        workspace_id,
        req.name,
        req.code,
        req.is_base_for_ot,
        req.is_taxable,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(component)))
}

pub async fn get_employee_salary_components(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    // Note: This returns latest components for effective_date = now
    let now = chrono::Utc::now().date_naive();
    let components = crate::repositories::payroll_repository::get_employee_salary_components(
        state.db.api(),
        employee_id,
        now,
    )
    .await?;

    // Flatten the tuple (Component, Definition) for response
    let response: Vec<_> = components
        .into_iter()
        .map(|(c, d)| {
            serde_json::json!({
                "id": c.id,
                "employee_id": c.employee_id,
                "component_definition_id": c.component_definition_id,
                "amount": c.amount,
                "effective_date": c.effective_date,
                "name": d.name,
                "code": d.code,
                "is_base_for_ot": d.is_base_for_ot,
                "is_taxable": d.is_taxable,
            })
        })
        .collect();

    Ok(Json(response))
}

pub async fn assign_employee_salary_component(
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
    Json(req): Json<AssignSalaryComponentRequest>,
) -> AppResult<(StatusCode, Json<EmployeeSalaryComponent>)> {
    let component = crate::repositories::payroll_repository::assign_employee_salary_component(
        state.db.api(),
        employee_id,
        req.component_definition_id,
        req.amount,
        req.effective_date,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(component)))
}
