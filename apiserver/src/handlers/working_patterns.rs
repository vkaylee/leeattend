use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    handlers::common::MemberContext,
    models::{CreateWorkingPatternRequest, AssignPatternRequest},
    services::working_pattern_service as service,
    AppState,
};

pub async fn create_pattern(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkingPatternRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    let pattern = service::create_working_pattern(&state.db, ctx.workspace_id, req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(pattern)))
}

pub async fn list_patterns(
    State(state): State<Arc<AppState>>,
    Path(_ws_id): Path<Uuid>,
    ctx: MemberContext,
) -> Result<impl IntoResponse, StatusCode> {
    let patterns = service::list_patterns(&state.db, ctx.workspace_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(patterns))
}

pub async fn get_pattern_details(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, pattern_id)): Path<(Uuid, Uuid)>,
    _ctx: MemberContext,
) -> Result<impl IntoResponse, StatusCode> {
    let details = service::get_pattern_details(&state.db, pattern_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(details))
}

pub async fn assign_pattern(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<AssignPatternRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    let assignment = service::assign_pattern(&state.db, ctx.workspace_id, employee_id, req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(assignment))
}

pub async fn get_employee_pattern_history(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
    _ctx: MemberContext,
) -> Result<impl IntoResponse, StatusCode> {
    let history = service::get_employee_pattern_history(&state.db, employee_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(history))
}
