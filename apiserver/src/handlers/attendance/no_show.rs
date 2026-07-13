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
    /// Calendar date to query (yyyy-MM-dd) in workspace-local time. Only shifts
    /// that overlap this day AND have already started (start < now for today;
    /// all shifts for past dates) are included.
    pub date: String,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    /// Comma-separated employee UUIDs to restrict the search to.
    pub employee_ids: Option<String>,
    pub department_id: Option<Uuid>,
    /// Comma-separated shift names to filter by shift type.
    #[serde(default)]
    pub shift_type: Option<String>,
}

/// No-show report: employees on a shift overlapping `date` who have not punched
/// in yet (status `absent`), filtered to only shifts that have *already started*.
/// For today, this means `shift_start < now`; for past dates, all shifts are
/// included (they have all started). Per-employee timezones are used to determine
/// "today" and shift boundaries.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/reports/no-show",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        NoShowQuery
    ),
    responses(
        (status = 200, description = "No-show employees with shifts overlapping the target date")
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

    // RBAC: requires at least read-level report access.
    ctx.require_permission(crate::domain::permissions::Permission::ReportsRead)?;

    let target_date = params
        .date
        .parse::<chrono::NaiveDate>()
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
            date: target_date,
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
