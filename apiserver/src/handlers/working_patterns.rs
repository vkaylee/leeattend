use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::{
    handlers::common::MemberContext,
    models::{AssignPatternRequest, BulkAssignPatternRequest, CreateWorkingPatternRequest, WorkingPattern, WorkingPatternDetail, EmployeeWorkingPattern, BulkAssignResponse},
    services::working_pattern_service as service,
    AppState,
};

pub async fn create_pattern(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkingPatternRequest>,
) -> AppResult<impl IntoResponse> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }
    let pattern = service::create_working_pattern(&state.db, ctx.workspace_id, req).await?;

    if let Some(cache) = &state.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok((StatusCode::CREATED, Json(pattern)))
}

pub async fn list_patterns(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<WorkingPattern>>> {
    let patterns = service::list_patterns(&state.db, ctx.workspace_id).await?;
    Ok(Json(patterns))
}

pub async fn get_pattern_details(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, pattern_id)): Path<(Uuid, Uuid)>,
    _ctx: MemberContext,
) -> AppResult<Json<Vec<WorkingPatternDetail>>> {
    let details = service::get_pattern_details(&state.db, pattern_id).await?;
    Ok(Json(details))
}

pub async fn assign_pattern(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<AssignPatternRequest>,
) -> AppResult<Json<EmployeeWorkingPattern>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }
    let assignment = service::assign_pattern(&state.db, ctx.workspace_id, employee_id, req).await?;

    if let Some(cache) = &state.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(Json(assignment))
}

pub async fn bulk_assign_pattern(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<BulkAssignPatternRequest>,
) -> AppResult<Json<BulkAssignResponse>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }
    let result = service::bulk_assign_pattern(&state.db, ctx.workspace_id, req).await?;

    if let Some(cache) = &state.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(Json(result))
}

pub async fn get_employee_pattern_history(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    _ctx: MemberContext,
) -> AppResult<Json<Vec<EmployeeWorkingPattern>>> {
    let history = service::get_employee_pattern_history(&state.db, employee_id).await?;
    Ok(Json(history))
}

pub async fn delete_pattern(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, pattern_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }
    service::delete_working_pattern(&state.db, pattern_id).await?;

    if let Some(cache) = &state.roster_cache {
        cache.invalidate(ctx.workspace_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}
