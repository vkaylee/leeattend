use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::dashboard_metrics::DashboardComparisonBasis;
use crate::models::{
    CreateRecruitmentSourceRequest, PaginatedResponse, PaginationParams, RecruitmentSource,
    RecruitmentSourceBreakdown, TurnoverAnalytics, TurnoverAnalyticsQuery, TurnoverTarget,
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

        truncated: None,
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
    ctx.require_permission(Permission::DashboardRead)?;
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
    ctx.require_permission(Permission::DashboardRead)?;
    let workspace_tz = state
        .core()
        .workspace_repo()
        .get_workspace_by_id(ctx.workspace_id)
        .await?
        .map(|ws| crate::utils::tz::parse_workspace_tz(ws.timezone()))
        .unwrap_or(chrono_tz::UTC);
    let today = crate::utils::tz::utc_now_date_in_tz(&workspace_tz);

    // Parse dates strictly — malformed input is a validation error, never a silent fallback.
    let start = match params.start {
        Some(s) => chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map_err(|_e| AppError::validation(crate::utils::i18n::I18nKey::InvalidDateFormat))?,
        None => today - chrono::Duration::days(180),
    };
    let end = match params.end {
        Some(s) => chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map_err(|_e| AppError::validation(crate::utils::i18n::I18nKey::InvalidDateFormat))?,
        None => today,
    };
    crate::models::validate_custom_range(start, end, today)?;

    let comparison_basis = match params.comparison.as_deref() {
        Some("previous_period") => DashboardComparisonBasis::PreviousPeriod,
        Some("same_period_last_year") => DashboardComparisonBasis::SamePeriodLastYear,
        Some("none") | None => DashboardComparisonBasis::None,
        Some(_other) => {
            return Err(AppError::validation(
                crate::utils::i18n::I18nKey::InvalidStatus,
            ))
        }
    };

    // Resolve the turnover target from workspace config, defaulting to 15% (lte).
    let configured_target = state
        .core()
        .workspace_config_service()
        .get_effective_decimal(&ctx.auth_token, "dashboard.budget.turnover_rate", today)
        .await?;
    let target = match params.target {
        Some(raw) => raw
            .parse::<rust_decimal::Decimal>()
            .map_err(|_e| AppError::validation(crate::utils::i18n::I18nKey::InvalidStatus))?,
        None => configured_target.unwrap_or_else(|| rust_decimal::Decimal::from(15)),
    };
    let target = TurnoverTarget {
        value: target,
        operator: "lte".to_string(),
        source: if configured_target.is_some() {
            "workspace_config".to_string()
        } else {
            "default".to_string()
        },
    };

    let query = TurnoverAnalyticsQuery {
        start_date: start,
        end_date: end,
        comparison_basis,
        target,
    };
    let analytics = state
        .employees()
        .recruitment_service()
        .get_turnover_analytics(&ctx.auth_token, &query)
        .await?;
    Ok(Json(analytics))
}

#[derive(Debug, serde::Deserialize)]
pub struct TurnoverQueryParams {
    pub start: Option<String>,
    pub end: Option<String>,
    /// `previous_period` | `same_period_last_year` | `none` (default `none`).
    pub comparison: Option<String>,
    /// Optional Decimal override for the turnover target (e.g. `"10"`).
    pub target: Option<String>,
}
