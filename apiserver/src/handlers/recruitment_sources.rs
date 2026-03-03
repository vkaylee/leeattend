use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    CreateRecruitmentSourceRequest, RecruitmentSource, RecruitmentSourceBreakdown,
    UpdateRecruitmentSourceRequest,
};
use crate::services::employee_service::EmployeeService;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_recruitment_sources(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<RecruitmentSource>>> {
    let sources = EmployeeService::get_recruitment_sources(&state.db, workspace_id)
        .await
        .map_err(|s| AppError::from((s, "Failed to get recruitment sources".to_string())))?;
    Ok(Json(sources))
}

pub async fn create_recruitment_source(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateRecruitmentSourceRequest>,
) -> AppResult<impl IntoResponse> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can manage recruitment sources".to_string(),
        ));
    }
    let source = EmployeeService::create_recruitment_source(&state.db, ctx.workspace_id, req)
        .await
        .map_err(|s| AppError::from((s, "Failed to create recruitment source".to_string())))?;
    Ok((StatusCode::CREATED, Json(source)))
}

pub async fn update_recruitment_source(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateRecruitmentSourceRequest>,
) -> AppResult<Json<RecruitmentSource>> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden(
            "Only admins can manage recruitment sources".to_string(),
        ));
    }
    let source =
        EmployeeService::update_recruitment_source(&state.db, id, ctx.workspace_id, req)
            .await
            .map_err(|s| {
                AppError::from((s, "Failed to update recruitment source".to_string()))
            })?;
    Ok(Json(source))
}

pub async fn delete_recruitment_source(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }
    EmployeeService::delete_recruitment_source(&state.db, id, ctx.workspace_id)
        .await
        .map_err(|s| AppError::from((s, "Failed to delete recruitment source".to_string())))?;
    Ok(StatusCode::NO_CONTENT)
}

/// Dashboard chart endpoint: recruitment source breakdown
pub async fn get_recruitment_source_breakdown(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<RecruitmentSourceBreakdown>>> {
    let breakdown =
        EmployeeService::get_recruitment_source_breakdown(&state.db, workspace_id)
            .await
            .map_err(|s| {
                AppError::from((s, "Failed to get recruitment source breakdown".to_string()))
            })?;
    Ok(Json(breakdown))
}
