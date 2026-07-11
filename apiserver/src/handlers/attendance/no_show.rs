use axum::{
    extract::{Query, State},
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::services::report_service::no_show::NoShowReportArgs;
use crate::utils::i18n::I18nKey;
use crate::AppState;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct NoShowQuery {
    /// Reference instant `A` (the scrubber's pinned center) as RFC3339 / ISO-8601
    /// in UTC, e.g. `2026-07-10T07:30:00Z`. The service widens this to the
    /// `[date(A)-1, date(A)]` window so overnight shifts are reachable.
    pub reference_time: String,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    /// Comma-separated employee UUIDs to restrict the search to.
    pub employee_ids: Option<String>,
    pub department_id: Option<Uuid>,
    /// Comma-separated shift names to filter by shift type.
    #[serde(default)]
    pub shift_type: Option<String>,
}

/// No-show report: employees on a shift *active* at `reference_time` who have
/// not punched in yet (status `absent`). Reached from the timeline scrubber.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/reports/no-show",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        NoShowQuery
    ),
    responses(
        (status = 200, description = "No-show employees active at the reference time")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip(state, ctx), fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn get_no_show_report(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(params): Query<NoShowQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let workspace_id = ctx.workspace_id;

    // RBAC: requires at least read-level report access, mirroring the daily
    // attendance report permission set.
    ctx.require_permission(crate::domain::permissions::Permission::ReportsRead)?;

    // Feature-flag gate: the timeline scrubber is released behind
    // `no_show_timeline`. The DB flag is the source of truth (see change-
    // management §6); an env override acts as an emergency kill-switch.
    let flag_enabled = state
        .payroll()
        .feature_flag_repo()
        .is_enabled_flag(&ctx.auth_token, "no_show_timeline")
        .await
        .unwrap_or(false);
    let env_enabled = std::env::var("FF_NO_SHOW_TIMELINE")
        .map(|v| v == "true")
        .unwrap_or(false);
    if !flag_enabled && !env_enabled {
        return Err(AppError::not_found(I18nKey::NotFound));
    }

    let reference_time = params
        .reference_time
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|_| AppError::validation(I18nKey::InvalidDateFormat))?;

    let employee_ids = params.employee_ids.as_ref().map(|s| {
        s.split(',')
            .filter_map(|id| Uuid::parse_str(id).ok())
            .collect::<Vec<Uuid>>()
    });

    let report = state
        .reports()
        .service()
        .get_no_show_report(NoShowReportArgs {
            workspace_id,
            reference_time,
            page: params.page,
            limit: params.limit,
            employee_ids,
            department_id: params.department_id,
            shift_type: params.shift_type,
        })
        .await?;

    Ok(Json(
        serde_json::to_value(report).map_err(|e| AppError::from(e.to_string()))?,
    ))
}
