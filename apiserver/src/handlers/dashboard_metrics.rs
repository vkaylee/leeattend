use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Json,
};
use std::sync::Arc;

use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{DashboardMetricsQuery, DashboardMetricsResponse};
use crate::services::audit_service::{AuditHelper, AuditLogArgs};
use crate::services::dashboard_metrics_service::{build_dashboard_metrics, BudgetConfig};
use crate::AppState;

/// Mask an email for audit logging: `user_***@domain.com`. Never log raw PII.
fn mask_email(email: &str) -> String {
    let (local, domain) = email.split_once('@').unwrap_or((email, ""));
    let prefix: String = local.chars().take(4).collect();
    let suffix = if domain.is_empty() {
        String::new()
    } else {
        format!("@{domain}")
    };
    format!("{prefix}_***{suffix}")
}

#[utoipa::path(
    get,
    path = "/dashboard/executive-overview",
    params(DashboardMetricsQuery),
    responses((status = 200, description = "Get workspace-scoped executive dashboard metrics", body = DashboardMetricsResponse))
)]
#[tracing::instrument(skip(state, ctx, headers), fields(workspace_id = %ctx.workspace_id), err)]
pub async fn get_dashboard_metrics(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    headers: HeaderMap,
    Query(query): Query<DashboardMetricsQuery>,
) -> AppResult<Json<DashboardMetricsResponse>> {
    ctx.require_permission(Permission::DashboardRead)?;
    let allow_overtime_cost = ctx.has_permission(Permission::PayrollSensitiveRead);
    let now = chrono::Utc::now();

    // Budget targets are workspace-scoped config; missing keys simply mean the
    // budget-variance metric reports `data_gap`.
    let budgets = if allow_overtime_cost {
        BudgetConfig {
            overtime_cost: state
                .core()
                .workspace_config_service()
                .get_effective_decimal(
                    &ctx.auth_token,
                    "dashboard.budget.overtime_cost",
                    now.date_naive(),
                )
                .await?,
            labor_cost: state
                .core()
                .workspace_config_service()
                .get_effective_decimal(
                    &ctx.auth_token,
                    "dashboard.budget.labor_cost",
                    now.date_naive(),
                )
                .await?,
        }
    } else {
        BudgetConfig::default()
    };

    let response = build_dashboard_metrics(
        state.core().workspace_repo(),
        state.reports().stats_repo(),
        &ctx.auth_token,
        &query,
        now,
        ctx.managed_department_ids.as_ref(),
        allow_overtime_cost,
        &budgets,
    )
    .await?;

    // Payroll-sensitive reads are audited when cost data was actually surfaced
    // (caller has the permission AND a paid/finalized payslip exists).
    let cost_emitted = allow_overtime_cost
        && response
            .metrics()
            .iter()
            .any(|metric| metric.key() == "overtime_cost" && metric.value().is_some());
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
                "period": response.context().period().key(),
                "start_date": response.context().period().start_date(),
                "end_date": response.context().period().end_date(),
                "currency": response.workspace().currency(),
                "cost_metrics": true,
            })),
            headers: &headers,
        })
        .await;
    }

    Ok(Json(response))
}
