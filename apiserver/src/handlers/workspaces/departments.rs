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

pub async fn get_departments(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<Department>>> {
    let departments =
        state.payroll.department_repo.get_departments(workspace_id)
            .await?;
    Ok(Json(departments))
}

pub async fn create_department(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateDepartmentRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::DepartmentsCreate)?;
    let dept = state.payroll.department_repo.create_department(ctx.workspace_id,
        req.name,
        req.description,
        req.manager_id,
        req.default_working_pattern_id,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(dept)))
}

pub async fn update_department(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_, id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> AppResult<Json<Department>> {
    ctx.require_permission(Permission::DepartmentsUpdate)?;
    let dept = state.payroll.department_repo.update_department(ctx.workspace_id,
        id,
        req.name,
        req.description,
        req.manager_id,
        req.default_working_pattern_id,
    )
    .await?;

    Ok(Json(dept))
}

pub async fn delete_department(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_, id)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::DepartmentsDelete)?;
    state.payroll.department_repo.delete_department(ctx.workspace_id, id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
