use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{CreateDepartmentRequest, Department, UpdateDepartmentRequest};
use crate::AppState;

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/departments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "List of departments", body = Vec<Department>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_departments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<Department>>> {
    let departments = state
        .payroll
        .department_repo
        .get_departments(&ctx.auth_token)
        .await?;
    Ok(Json(departments))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/departments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateDepartmentRequest,
    responses(
        (status = 201, description = "Department created", body = Department)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_department(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateDepartmentRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::DepartmentsCreate)?;
    let dept = state
        .payroll
        .department_repo
        .create_department(
            &ctx.auth_token,
            req.name,
            req.description,
            req.manager_id,
            req.default_working_pattern_id,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(dept)))
}

#[utoipa::path(
    put,
    path = "/v1/workspaces/{ws_id}/departments/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Department ID")
    ),
    request_body = UpdateDepartmentRequest,
    responses(
        (status = 200, description = "Department updated", body = Department)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_department(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> AppResult<Json<Department>> {
    let (_, id) = __path_params;
    ctx.require_permission(Permission::DepartmentsUpdate)?;
    let dept = state
        .payroll
        .department_repo
        .update_department(&ctx.auth_token, id, req)
        .await?;

    Ok(Json(dept))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/departments/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Department ID to delete")
    ),
    responses(
        (status = 204, description = "Department deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_department(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    let (_, id) = __path_params;
    ctx.require_permission(Permission::DepartmentsDelete)?;
    state
        .payroll
        .department_repo
        .delete_department(&ctx.auth_token, id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
