use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    CreateRecruitmentSourceRequest, RecruitmentSource, RecruitmentSourceBreakdown,
    UpdateRecruitmentSourceRequest,
};
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
    let sources = state
        .employee_service
        .get_recruitment_sources(workspace_id)
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
    ctx.require_permission(Permission::RecruitmentManage)?;
    let source = state
        .employee_service
        .create_recruitment_source(ctx.workspace_id, req)
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
    ctx.require_permission(Permission::RecruitmentManage)?;
    let source = state
        .employee_service
        .update_recruitment_source(id, ctx.workspace_id, req)
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
    ctx.require_permission(Permission::RecruitmentManage)?;
    state
        .employee_service
        .delete_recruitment_source(id, ctx.workspace_id)
        .await
        .map_err(|s| AppError::from((s, "Failed to delete recruitment source".to_string())))?;
    Ok(StatusCode::NO_CONTENT)
}

/// Dashboard chart endpoint: recruitment source breakdown
pub async fn get_recruitment_source_breakdown(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<RecruitmentSourceBreakdown>>> {
    let breakdown = state
        .employee_service
        .get_recruitment_source_breakdown(workspace_id)
        .await
        .map_err(|s| {
            AppError::from((s, "Failed to get recruitment source breakdown".to_string()))
        })?;
    Ok(Json(breakdown))
}

/// Dashboard chart endpoint: turnover analytics with date range filter
pub async fn get_turnover_analytics(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    axum::extract::Query(params): axum::extract::Query<TurnoverQueryParams>,
) -> AppResult<Json<crate::models::TurnoverAnalytics>> {
    let today = chrono::Utc::now().date_naive();
    let start = params
        .start
        .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| today - chrono::Duration::days(180));
    let end = params
        .end
        .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .unwrap_or(today);

    let analytics = state
        .employee_service
        .get_turnover_analytics(workspace_id, start, end)
        .await
        .map_err(|s| AppError::from((s, "Failed to get turnover analytics".to_string())))?;
    Ok(Json(analytics))
}

#[derive(Debug, serde::Deserialize)]
pub struct TurnoverQueryParams {
    pub start: Option<String>,
    pub end: Option<String>,
}
