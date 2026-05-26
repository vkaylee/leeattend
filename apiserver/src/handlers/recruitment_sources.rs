use crate::domain::permissions::Permission;
use crate::errors::AppResult;
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
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<RecruitmentSource>>> {
    let _workspace_id = ctx.workspace_id;

    let sources = state
        .employees
        .service
        .get_recruitment_sources(&ctx.auth_token)
        .await?;
    Ok(Json(sources))
}

pub async fn create_recruitment_source(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateRecruitmentSourceRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::RecruitmentManage)?;
    let source = state
        .employees
        .service
        .create_recruitment_source(&ctx.auth_token, req)
        .await?;
    Ok((StatusCode::CREATED, Json(source)))
}

pub async fn update_recruitment_source(
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateRecruitmentSourceRequest>,
) -> AppResult<Json<RecruitmentSource>> {
    ctx.require_permission(Permission::RecruitmentManage)?;
    let source = state
        .employees
        .service
        .update_recruitment_source(id, &ctx.auth_token, req)
        .await?;
    Ok(Json(source))
}

pub async fn delete_recruitment_source(
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::RecruitmentManage)?;
    state
        .employees
        .service
        .delete_recruitment_source(id, &ctx.auth_token)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Dashboard chart endpoint: recruitment source breakdown
pub async fn get_recruitment_source_breakdown(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<RecruitmentSourceBreakdown>>> {
    let _workspace_id = ctx.workspace_id;

    let breakdown = state
        .employees
        .service
        .get_recruitment_source_breakdown(&ctx.auth_token)
        .await?;
    Ok(Json(breakdown))
}

/// Dashboard chart endpoint: turnover analytics with date range filter
pub async fn get_turnover_analytics(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    axum::extract::Query(params): axum::extract::Query<TurnoverQueryParams>,
) -> AppResult<Json<crate::models::TurnoverAnalytics>> {
    let _workspace_id = ctx.workspace_id;

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
        .employees
        .service
        .get_turnover_analytics(&ctx.auth_token, start, end)
        .await?;
    Ok(Json(analytics))
}

#[derive(Debug, serde::Deserialize)]
pub struct TurnoverQueryParams {
    pub start: Option<String>,
    pub end: Option<String>,
}
