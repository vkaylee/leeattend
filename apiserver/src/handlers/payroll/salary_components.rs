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

/// GET /workspaces/{ws_id}/salary-components
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/salary-components",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Workspace salary components list", body = [SalaryComponentDefinition])
    )
)]
pub async fn get_salary_components(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<SalaryComponentDefinition>>> {
    let _workspace_id = ctx.workspace_id;

    let components = state
        .payroll
        .repo
        .get_salary_component_definitions(ctx.workspace_id)
        .await?;

    Ok(Json(components))
}

/// POST /workspaces/{ws_id}/salary-components
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/salary-components",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateSalaryComponentRequest,
    responses(
        (status = 201, description = "Created salary component", body = SalaryComponentDefinition)
    )
)]
pub async fn create_salary_component(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateSalaryComponentRequest>,
) -> AppResult<(StatusCode, Json<SalaryComponentDefinition>)> {
    let _workspace_id = ctx.workspace_id;

    let component = state
        .payroll
        .repo
        .create_salary_component_definition(
            ctx.workspace_id,
            req.name,
            req.code,
            req.base_group_ids,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(component)))
}

/// GET /workspaces/{ws_id}/employees/{employee_id}/salary-components
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/employees/{employee_id}/salary-components",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID")
    ),
    responses(
        (status = 200, description = "Employee salary components list", body = [Value])
    )
)]
pub async fn get_employee_salary_components(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    let (_, employee_id) = params;
    let _workspace_id = ctx.workspace_id;

    // Note: This returns latest components for effective_date = now
    let now = chrono::Utc::now().date_naive();
    let components = state
        .payroll
        .repo
        .get_employee_salary_components(ctx.workspace_id, employee_id, now)
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
            })
        })
        .collect();

    Ok(Json(response))
}

/// POST /workspaces/{ws_id}/employees/{employee_id}/salary-components
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/employees/{employee_id}/salary-components",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID")
    ),
    request_body = AssignSalaryComponentRequest,
    responses(
        (status = 201, description = "Assigned salary component to employee", body = EmployeeSalaryComponent)
    )
)]
pub async fn assign_employee_salary_component(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignSalaryComponentRequest>,
) -> AppResult<(StatusCode, Json<EmployeeSalaryComponent>)> {
    let (_, employee_id) = params;
    let workspace_id = ctx.workspace_id;

    let component = state
        .payroll
        .repo
        .assign_employee_salary_component(
            workspace_id,
            employee_id,
            req.component_definition_id,
            req.amount,
            req.effective_date,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(component)))
}
