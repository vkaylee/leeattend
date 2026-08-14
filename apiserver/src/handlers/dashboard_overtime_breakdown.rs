use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::handlers::workspaces::crud::ChartRangeQuery;
use crate::models::{validate_custom_range, OvertimeBreakdownRow};
use crate::repositories::workspace_stats_repository::ChartAnalyticsQuery;
use crate::services::audit_service::{AuditHelper, AuditLogArgs};
use crate::utils::masking::mask_email;
use crate::AppState;

/// Query params for `GET /v1/dashboard/overtime-breakdown`.
#[derive(Debug, Deserialize)]
pub struct OvertimeBreakdownQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub timezone: Option<String>,
    /// Optional Top-N cap on departments (sorted by system OT hours desc). Clamped to 1..=100.
    pub top_n: Option<i64>,
}

/// Workspace + currency context attached to the breakdown response.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OvertimeBreakdownWorkspace {
    pub timezone: String,
    pub currency: String,
}

/// Envelope for the department-level overtime breakdown.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OvertimeBreakdownResponse {
    pub contract_version: String,
    pub workspace: OvertimeBreakdownWorkspace,
    pub rows: Vec<OvertimeBreakdownRow>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[utoipa::path(
    get,
    path = "/dashboard/overtime-breakdown",
    params(
        ("start_date" = Option<String>, Query, description = "Inclusive start date (YYYY-MM-DD)"),
        ("end_date" = Option<String>, Query, description = "Inclusive end date (YYYY-MM-DD)"),
        ("timezone" = Option<String>, Query, description = "IANA timezone for date bucketing"),
        ("top_n" = Option<i64>, Query, description = "Return at most N departments sorted by system OT hours")
    ),
    responses(
        (status = 200, description = "Department-level overtime hours & cost breakdown", body = OvertimeBreakdownResponse)
    )
)]
#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip(state, ctx, headers), fields(workspace_id = %ctx.workspace_id), err)]
pub async fn get_overtime_breakdown(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    headers: HeaderMap,
    Query(params): Query<OvertimeBreakdownQuery>,
) -> AppResult<Json<OvertimeBreakdownResponse>> {
    ctx.require_permission(Permission::DashboardRead)?;
    let allow_overtime_cost = ctx.has_permission(Permission::PayrollSensitiveRead);

    // Resolve workspace timezone + currency for the response context.
    let workspace = state
        .core()
        .workspace_repo()
        .get_workspace_by_id(ctx.workspace_id)
        .await?
        .ok_or_else(|| {
            crate::errors::AppError::not_found(crate::utils::i18n::I18nKey::WorkspaceNotFound)
        })?;
    let default_tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());

    let (start_date, end_date, timezone) =
        match (params.start_date.as_deref(), params.end_date.as_deref()) {
            (None, None) => {
                let today = crate::utils::tz::utc_now_date_in_tz(&default_tz);
                (
                    today - chrono::Duration::days(29),
                    today,
                    params.timezone.clone(),
                )
            }
            (Some(_), Some(_)) => {
                let resolved = ChartRangeQuery {
                    start_date: params.start_date,
                    end_date: params.end_date,
                    timezone: params.timezone,
                }
                .resolve(default_tz)?;
                if let Some((start, end, tz)) = resolved {
                    (start, end, tz)
                } else {
                    let today = crate::utils::tz::utc_now_date_in_tz(&default_tz);
                    (today - chrono::Duration::days(29), today, None)
                }
            }
            _ => {
                return Err(crate::errors::AppError::validation(
                    crate::utils::i18n::I18nKey::InvalidDateRange,
                ));
            }
        };

    // Validate window + scope (strict, no silent fallback).
    let today = crate::utils::tz::utc_now_date_in_tz(&default_tz);
    validate_custom_range(start_date, end_date, today)?;

    let top_n = params.top_n.map(|n| n.clamp(1, 100));

    // An empty managed set means the member has no department scope at all —
    // treat it as global (None), matching the executive dashboard. Passing
    // `Some([])` would make `= ANY('{}')` false for every row and hide all data.
    let department_ids = ctx
        .managed_department_ids
        .as_ref()
        .filter(|ids| !ids.is_empty())
        .map(|ids| ids.iter().copied().collect::<Vec<_>>());

    let query = ChartAnalyticsQuery {
        department_ids,
        start_date: Some(start_date),
        end_date: Some(end_date),
        timezone,
        top_n,
    };

    let breakdown = state
        .reports()
        .stats_repo()
        .fetch_overtime_breakdown(&ctx.auth_token, &query, allow_overtime_cost)
        .await?;

    let currency = breakdown
        .currency_code
        .as_deref()
        .filter(|value| value.len() == 3)
        .unwrap_or("USD")
        .to_string();

    let mut rows = breakdown.rows;
    if let Some(limit) = top_n {
        // top_n is clamped to 1..=100, so the lossless cast to usize is safe.
        rows.truncate(usize::try_from(limit).unwrap_or(0));
    }

    let generated_at = chrono::Utc::now();

    // Payroll-sensitive cost reads are audited when the caller has the permission
    // AND at least one paid/finalized payslip contributed to the breakdown.
    let cost_emitted = allow_overtime_cost && rows.iter().any(|row| row.cost_available);
    if cost_emitted {
        AuditHelper::log(AuditLogArgs {
            repo: state.core().audit_repo(),
            workspace_id: ctx.workspace_id,
            actor_id: Some(ctx.user_id),
            actor_type: Some("user".to_string()),
            actor_name: ctx.user_name.clone(),
            actor_email: Some(mask_email(&ctx.user_email)),
            target_user_id: None,
            target_resource_id: None,
            action: "dashboard.sensitive.read",
            details: Some(serde_json::json!({
                "endpoint": "overtime-breakdown",
                "start_date": start_date.to_string(),
                "end_date": end_date.to_string(),
                "currency": currency,
                "cost_metrics": true,
            })),
            headers: &headers,
        })
        .await;
    }

    Ok(Json(OvertimeBreakdownResponse {
        contract_version: "overtime-breakdown.v1".to_string(),
        workspace: OvertimeBreakdownWorkspace {
            timezone: default_tz.name().to_string(),
            currency,
        },
        rows,
        generated_at,
    }))
}
