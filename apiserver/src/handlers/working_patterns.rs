use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::{
    handlers::common::MemberContext,
    models::{
        AssignPatternRequest, BulkAssignPatternRequest, BulkAssignResponse,
        CreateWorkingPatternRequest, EmployeeWorkingPattern, WorkingPattern, WorkingPatternDetail,
    },
    services::working_pattern_service as service,
    AppState,
};

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/working-patterns",
    request_body = CreateWorkingPatternRequest,
    responses(
        (status = 201, description = "Working pattern created", body = WorkingPattern)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn create_pattern(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkingPatternRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::PatternsCreate)?;
    let pattern = service::create_working_pattern(
        Arc::clone(state.shifts().working_pattern_repo()),
        Arc::clone(state.shifts().shift_repo()),
        ctx.workspace_id,
        req,
    )
    .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok((StatusCode::CREATED, Json(pattern)))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/working-patterns",
    responses(
        (status = 200, description = "List of working patterns", body = [WorkingPattern])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn list_patterns(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<WorkingPattern>>> {
    let patterns = service::list_patterns(
        Arc::clone(state.shifts().working_pattern_repo()),
        ctx.workspace_id,
    )
    .await?;
    Ok(Json(patterns))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/working-patterns/{pattern_id}",
    responses(
        (status = 200, description = "Working pattern details", body = [WorkingPatternDetail])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("pattern_id" = Uuid, Path, description = "Pattern ID")
    )
)]
pub async fn get_pattern_details(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<WorkingPatternDetail>>> {
    let (_ws_id, pattern_id) = params;
    let details = service::get_pattern_details(
        Arc::clone(state.shifts().working_pattern_repo()),
        ctx.workspace_id,
        pattern_id,
    )
    .await?;
    Ok(Json(details))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/employees/{id}/working-patterns",
    request_body = AssignPatternRequest,
    responses(
        (status = 200, description = "Pattern assigned to employee", body = EmployeeWorkingPattern)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID")
    )
)]
pub async fn assign_pattern(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<AssignPatternRequest>,
) -> AppResult<Json<EmployeeWorkingPattern>> {
    let (_ws_id, employee_id) = params;
    ctx.require_permission(Permission::PatternsAssign)?;
    let assignment = service::assign_pattern(
        Arc::clone(state.shifts().working_pattern_repo()),
        ctx.workspace_id,
        employee_id,
        req,
    )
    .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(Json(assignment))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/working-patterns/bulk-assign",
    request_body = BulkAssignPatternRequest,
    responses(
        (status = 200, description = "Bulk pattern assignments completed", body = BulkAssignResponse)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn bulk_assign_pattern(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<BulkAssignPatternRequest>,
) -> AppResult<Json<BulkAssignResponse>> {
    ctx.require_permission(Permission::PatternsAssign)?;
    let result = service::bulk_assign_pattern(
        Arc::clone(state.shifts().working_pattern_repo()),
        ctx.workspace_id,
        req,
    )
    .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/employees/{id}/working-patterns",
    responses(
        (status = 200, description = "Employee working pattern history", body = [EmployeeWorkingPattern])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID")
    )
)]
pub async fn get_employee_pattern_history(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<EmployeeWorkingPattern>>> {
    let (_ws_id, employee_id) = params;
    let history = service::get_employee_pattern_history(
        Arc::clone(state.shifts().working_pattern_repo()),
        ctx.workspace_id,
        employee_id,
    )
    .await?;
    Ok(Json(history))
}

#[utoipa::path(
    delete,
    path = "/workspaces/{workspace_id}/working-patterns/{pattern_id}",
    responses(
        (status = 204, description = "Working pattern deleted")
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("pattern_id" = Uuid, Path, description = "Pattern ID")
    )
)]
pub async fn delete_pattern(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_ws_id, pattern_id) = params;
    ctx.require_permission(Permission::PatternsDelete)?;
    service::delete_working_pattern(
        Arc::clone(state.shifts().working_pattern_repo()),
        ctx.workspace_id,
        pattern_id,
    )
    .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/workspaces/{workspace_id}/working-patterns/{pattern_id}",
    request_body = CreateWorkingPatternRequest,
    responses(
        (status = 200, description = "Working pattern updated", body = WorkingPattern)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("pattern_id" = Uuid, Path, description = "Pattern ID")
    )
)]
pub async fn update_pattern(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkingPatternRequest>,
) -> AppResult<Json<WorkingPattern>> {
    let (_ws_id, pattern_id) = params;
    ctx.require_permission(Permission::PatternsCreate)?;
    let pattern = service::update_working_pattern(
        Arc::clone(state.shifts().working_pattern_repo()),
        Arc::clone(state.shifts().shift_repo()),
        ctx.workspace_id,
        pattern_id,
        req,
    )
    .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(Json(pattern))
}
