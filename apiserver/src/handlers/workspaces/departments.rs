use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{CreateDepartmentRequest, Department, UpdateDepartmentRequest};
use crate::AppState;

pub async fn get_departments(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<Department>>> {
    let departments =
        crate::repositories::department_repository::get_departments(state.db.api(), workspace_id)
            .await?;
    Ok(Json(departments))
}

pub async fn create_department(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateDepartmentRequest>,
) -> AppResult<impl IntoResponse> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can manage departments".to_string(),
        ));
    }
    let dept = crate::repositories::department_repository::create_department(
        state.db.api(),
        ctx.workspace_id,
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
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> AppResult<Json<Department>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can manage departments".to_string(),
        ));
    }
    let dept = crate::repositories::department_repository::update_department(
        state.db.api(),
        ctx.workspace_id,
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
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can manage departments".to_string(),
        ));
    }
    crate::repositories::department_repository::delete_department(state.db.api(), ctx.workspace_id, id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
