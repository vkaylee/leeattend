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
        .payroll()
        .repo()
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
        .payroll()
        .repo()
        .create_salary_component_definition(
            ctx.workspace_id,
            req.name,
            req.code,
            req.base_group_ids,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(component)))
}

/// GET /workspaces/{ws_id}/employments/{employment_id}/salary-components
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/employments/{employment_id}/salary-components",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employment_id" = Uuid, Path, description = "Employment ID")
    ),
    responses(
        (status = 200, description = "Employment salary components list", body = [Value])
    )
)]
pub async fn get_employment_salary_components(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    let (_, employment_id) = params;
    let _workspace_id = ctx.workspace_id;

    let workspace_tz = state
        .core()
        .workspace_repo()
        .get_workspace_by_id(ctx.workspace_id)
        .await?
        .map(|ws| crate::utils::tz::parse_workspace_tz(ws.timezone()))
        .unwrap_or(chrono_tz::UTC);

    // Note: This returns latest components for effective_date = now
    let now = crate::utils::tz::utc_now_date_in_tz(&workspace_tz);
    let components = state
        .payroll()
        .repo()
        .get_employment_salary_components(ctx.workspace_id, employment_id, now)
        .await?;

    // Flatten the tuple (Component, Definition) for response
    let response: Vec<_> = components
        .into_iter()
        .map(|(c, d)| {
            serde_json::json!({
                "id": c.id,
                "employment_id": c.employment_id,
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

/// POST /workspaces/{ws_id}/employments/{employment_id}/salary-components
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/employments/{employment_id}/salary-components",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employment_id" = Uuid, Path, description = "Employment ID")
    ),
    request_body = AssignSalaryComponentRequest,
    responses(
        (status = 201, description = "Assigned salary component to employment", body = EmployeeSalaryComponent)
    )
)]
pub async fn assign_employment_salary_component(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignSalaryComponentRequest>,
) -> AppResult<(StatusCode, Json<EmployeeSalaryComponent>)> {
    let (_, employment_id) = params;
    let workspace_id = ctx.workspace_id;

    let component = state
        .payroll()
        .repo()
        .assign_employment_salary_component(
            workspace_id,
            employment_id,
            req.component_definition_id,
            req.amount,
            req.effective_date,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(component)))
}
