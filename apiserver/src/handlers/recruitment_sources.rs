use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{
    CreateRecruitmentSourceRequest, PaginatedResponse, PaginationParams, RecruitmentSource,
    RecruitmentSourceBreakdown, TurnoverAnalytics, UpdateRecruitmentSourceRequest,
};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedRecruitmentSources {
    pub data: Vec<RecruitmentSource>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/recruitment-sources",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        PaginationParams
    ),
    responses(
        (status = 200, description = "List of recruitment sources", body = PaginatedRecruitmentSources)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_recruitment_sources(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    axum::extract::Query(query): axum::extract::Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<RecruitmentSource>>> {
    let _workspace_id = ctx.workspace_id;
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (sources, total) = state
        .employees()
        .recruitment_service()
        .get_recruitment_sources(&ctx.auth_token, page, limit)
        .await?;
    Ok(Json(PaginatedResponse {
        data: sources,
        total: usize::try_from(total).unwrap_or(0),
        page,
        limit,
    }))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/recruitment-sources",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateRecruitmentSourceRequest,
    responses(
        (status = 201, description = "Recruitment source created", body = RecruitmentSource)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_recruitment_source(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateRecruitmentSourceRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::RecruitmentManage)?;
    let source = state
        .employees()
        .recruitment_service()
        .create_recruitment_source(&ctx.auth_token, req)
        .await?;
    Ok((StatusCode::CREATED, Json(source)))
}

#[utoipa::path(
    patch,
    path = "/v1/workspaces/{ws_id}/recruitment-sources/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Source ID")
    ),
    request_body = UpdateRecruitmentSourceRequest,
    responses(
        (status = 200, description = "Recruitment source updated", body = RecruitmentSource)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_recruitment_source(
    Path(__path_params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateRecruitmentSourceRequest>,
) -> AppResult<Json<RecruitmentSource>> {
    let (_, id) = __path_params;
    ctx.require_permission(Permission::RecruitmentManage)?;
    let source = state
        .employees()
        .recruitment_service()
        .update_recruitment_source(id, &ctx.auth_token, req)
        .await?;
    Ok(Json(source))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/recruitment-sources/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Source ID")
    ),
    responses(
        (status = 204, description = "Recruitment source deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_recruitment_source(
    Path(__path_params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_, id) = __path_params;
    ctx.require_permission(Permission::RecruitmentManage)?;
    state
        .employees()
        .recruitment_service()
        .delete_recruitment_source(id, &ctx.auth_token)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Dashboard chart endpoint: recruitment source breakdown
#[utoipa::path(
    get,
    path = "/v1/dashboard/recruitment-sources",
    responses(
        (status = 200, description = "Recruitment source breakdown", body = Vec<RecruitmentSourceBreakdown>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_recruitment_source_breakdown(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<RecruitmentSourceBreakdown>>> {
    let _workspace_id = ctx.workspace_id;

    let breakdown = state
        .employees()
        .recruitment_service()
        .get_recruitment_source_breakdown(&ctx.auth_token)
        .await?;
    Ok(Json(breakdown))
}

/// Dashboard chart endpoint: turnover analytics with date range filter
#[utoipa::path(
    get,
    path = "/v1/dashboard/turnover-analytics",
    params(
        ("start" = Option<String>, Query, description = "Start date (YYYY-MM-DD)"),
        ("end" = Option<String>, Query, description = "End date (YYYY-MM-DD)")
    ),
    responses(
        (status = 200, description = "Turnover analytics", body = TurnoverAnalytics)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_turnover_analytics(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    axum::extract::Query(params): axum::extract::Query<TurnoverQueryParams>,
) -> AppResult<Json<crate::models::TurnoverAnalytics>> {
    let workspace_tz = state
        .core()
        .workspace_repo()
        .get_workspace_by_id(ctx.workspace_id)
        .await?
        .map(|ws| crate::utils::tz::parse_workspace_tz(ws.timezone()))
        .unwrap_or(chrono_tz::UTC);
    let today = crate::utils::tz::utc_now_date_in_tz(&workspace_tz);
    let start = params
        .start
        .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .unwrap_or_else(|| today - chrono::Duration::days(180));
    let end = params
        .end
        .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .unwrap_or(today);

    let analytics = state
        .employees()
        .recruitment_service()
        .get_turnover_analytics(&ctx.auth_token, start, end)
        .await?;
    Ok(Json(analytics))
}

#[derive(Debug, serde::Deserialize)]
pub struct TurnoverQueryParams {
    pub start: Option<String>,
    pub end: Option<String>,
}
