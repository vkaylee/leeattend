use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{CreateDepartmentRequest, UpdateDepartmentRequest};
use crate::AppState;

pub async fn get_departments(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::department_repository::get_departments(&state.db, workspace_id).await
    {
        Ok(departments) => Json(departments).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_department(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateDepartmentRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage departments").into_response();
    }
    match crate::repositories::department_repository::create_department(
        &state.db,
        ctx.workspace_id,
        req.name,
        req.description,
        req.manager_id,
        req.default_working_pattern_id,
    )
    .await
    {
        Ok(dept) => (StatusCode::CREATED, Json(dept)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_department(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage departments").into_response();
    }
    match crate::repositories::department_repository::update_department(
        &state.db,
        ctx.workspace_id,
        id,
        req.name,
        req.description,
        req.manager_id,
        req.default_working_pattern_id,
    )
    .await
    {
        Ok(dept) => Json(dept).into_response(),
        Err(e) => {
            tracing::error!("Failed to update department: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn delete_department(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage departments").into_response();
    }
    match crate::repositories::department_repository::delete_department(
        &state.db,
        ctx.workspace_id,
        id,
    )
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
